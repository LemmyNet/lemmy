-- prototype/draft, work in progress

-- meaning of the name "post/comment byebye and return"
--   post/comment can be deleted or removed or banned by community or community itself is banned or hidden = byebye
--   byebye actions can be reversed = return
--   post or comment present many of the same logic choices.

-- lemmy_server 0.18.2 context:
--   byebye actions are causing massive performance problems on the site. Individual deletes taking PostgreSQL many seconds.
--   byebye actions upon multiple posts and comments, such as account delete, are crashing the servers.


-- ASSUMPTIONS:
--    1. calling trigger already confirmed deleted OR removed field change
--    2. (clarified 1) ???
--    3. lemmy Rust code does not touch comments when a post is deleted/removed (or reverse of same)
--    4. this function is called only on UPDATE, not SQL DELETE statements. Heads up:
--          Because a DELETE would need to consider if it was previously deleted/removed and count decremented.
--    5. it is possible for both user delete and mod remove to happen to the same post
--       this means to consider not decrementing or incrementing twice for one actual post.
--    6. SET deleted = 't' SET removed = 't' do not come in on the same SQL UPDATE, they are unique
--           SQL UPDATE statements (and client API calls) within the Lemmy Rust code.
--    7. community_id of a post isn't being changed at same time, NEW is stable on this field
--    8. creator_id of a post isn't being changed at same time, NEW is stable on this field
--    9. an INSERT trigger would not be called to match this function because INSERT
--           of an already-deleted or already-removed post makes no sense?
create or replace function local_aggregates_existing_post_change_count()
    returns trigger language plpgsql
as $$
begin
    DECLARE
        -- start out assuming this is a user deleting or mod removing.
        -- restores are rare, deletes of posts routine.
        post_change integer := -1;
        comment_change integer := 0;
        prev_post_aggregate RECORD; -- previous post aggregate record
        
    -- eliminating decrementing or incrementing twice for the same post end results
    --    end-user has already deleted the post, counts already decremented
    --    moderator is removing
    --    moderator is restoring
    IF (NEW.deleted = 't' AND OLD.removed != NEW.removed) THEN RETURN; END IF;
    --    moderator has already removed the post, counts already decremented
    --    end user is deleting
    --    end user is undeleting
    IF (NEW.removed = 't' AND OLD.deleted != NEW.deleted) THEN RETURN; END IF;

    -- posts own comments, so big changes to comment counts on decrement or restore-increment
       -- can we pull the count from post_aggregates as a single SQL SELECT row?
       -- can we add another assumption, that there are no SQL delete row triggers on post_aggregates
       --    until after we have revised counts?
    -- THIS next statement isn't complete, returns negative value intentionally
    -- comment_change = SELECT 0 - comments FROM post_aggregates WHERE post_id = NEW.post_id
    prev_post_aggregate = SELECT * FROM post_aggregates WHERE post_id = NEW.post_id
    comment_change = 0 - prev_post_aggregate.comments;
    -- id post_id comments score upvotes downvotes published newest_comment_time_necro newest_comment_time featured_community featured_local hot_rank hot_rank_active

    -- is this a reversal of a previous user delete or mod remove?
    IF (OLD.deleted = 't' AND NEW.deleted = 'f') THEN
        post_change := 1;
        comment_change := 0 - comment_change;
        -- comment_change := prev_post_aggregate.comments;
    END IF;
    IF (OLD.removed = 't' AND NEW.removed = 'f') THEN
        post_change := 1;
        comment_change := 0 - comment_change;
        -- comment_change := prev_post_aggregate.comments;
    END IF;
    

    -- BEGIN TRANSACTION

    update site_aggregates
    set posts = posts + post_change, comments = comments + comment_change
    -- site_id 1 is hard-coded known value for local site
    site_id = 1;

    update community_aggregates
    set posts = posts + post_change, comments = comments + comment_change
    where community_id = NEW.community_id;

    update person_aggregates
    set post_count = post_count + post_change, comments = comments + comment_change
    where person_id = NEW.creator_id;

    -- END TRANSACTION
end $$;


-- ToDo: mass update happens with lemmy account delete, perhaps "for each statement" design
-- user can delete or undelete a post (deleted column)
-- moderator can remove or restore a post (removed column)
create trigger local_aggregates_post_user_or_mod_update_count
    after update of removed, deleted
    on post
    for each row
        when (
            (OLD.local = true)
            -- filter out possible duplicate SQL actions by client, such as delete twice of same post
            -- assumption made that a single Lemmy Rust UPDATE is not an end-user delete and mod-remove in one statement
            -- assumpiton made that a single Lemmy Rust UPDATE is not an end-user undelete and mod-restore in one statement
            and (
                (OLD.removed is distinct from NEW.removed)
                OR 
                (OLD.deleted is distinct from NEW.deleted)
                )
            )
        execute procedure local_aggregates_existing_post_change_count();

