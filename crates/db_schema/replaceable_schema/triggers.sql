-- A trigger is associated with a table instead of a schema, so they can't be in the `r` schema. This is
-- okay if the function specified after `EXECUTE FUNCTION` is in `r`, since dropping the function drops the trigger.
--
-- Tables that are updated by triggers should not have foreign keys that aren't set to `INITIALLY DEFERRED`
-- (even if only other columns are updated) because triggers can run after the deletion of referenced rows and
-- before the automatic deletion of the row that references it. This is not a problem for insert or delete.
--
-- Triggers that update multiple tables should use this order: person_aggregates, comment_aggregates,
-- post_aggregates, community_aggregates, site_aggregates
--   * The order matters because the updated rows are locked until the end of the transaction, and statements
--     in a trigger don't use separate transactions. This means that updates closer to the beginning cause
--     longer locks because the duration of each update extends the durations of the locks caused by previous
--     updates. Long locks are worse on rows that have more concurrent transactions trying to update them. The
--     listed order starts with tables that are less likely to have such rows.
--     https://www.postgresql.org/docs/16/transaction-iso.html#XACT-READ-COMMITTED
--   * Using the same order in every trigger matters because a deadlock is possible if multiple transactions
--     update the same rows in a different order.
--     https://www.postgresql.org/docs/current/explicit-locking.html#LOCKING-DEADLOCKS
--
--
-- Create triggers for both post and comments
CREATE FUNCTION r.creator_id_from_post_aggregates (agg post_aggregates)
    RETURNS int IMMUTABLE PARALLEL SAFE RETURN agg.creator_id;

CREATE FUNCTION r.creator_id_from_comment_aggregates (agg comment_aggregates)
    RETURNS int IMMUTABLE PARALLEL SAFE RETURN (
        SELECT
            creator_id
        FROM
            comment
        WHERE
            comment.id = agg.comment_id LIMIT 1
);

CREATE PROCEDURE r.post_or_comment (table_name text)
LANGUAGE plpgsql
AS $a$
BEGIN
    EXECUTE replace($b$
        -- When a thing gets a vote, update its aggregates and its creator's aggregates
        CALL r.create_triggers ('thing_like', $$
            BEGIN
                WITH thing_diff AS ( UPDATE
                        thing_aggregates AS a
                    SET
                        score = a.score + diff.upvotes - diff.downvotes, upvotes = a.upvotes + diff.upvotes, downvotes = a.downvotes + diff.downvotes, controversy_rank = r.controversy_rank ((a.upvotes + diff.upvotes)::numeric, (a.downvotes + diff.downvotes)::numeric)
                    FROM (
                        SELECT
                            (thing_like).thing_id, coalesce(sum(count_diff) FILTER (WHERE (thing_like).score = 1), 0) AS upvotes, coalesce(sum(count_diff) FILTER (WHERE (thing_like).score != 1), 0) AS downvotes FROM select_old_and_new_rows AS old_and_new_rows GROUP BY (thing_like).thing_id) AS diff
            WHERE
                a.thing_id = diff.thing_id
                    AND (diff.upvotes, diff.downvotes) != (0, 0)
                RETURNING
                    r.creator_id_from_thing_aggregates (a.*) AS creator_id, diff.upvotes - diff.downvotes AS score)
            UPDATE
                person_aggregates AS a
            SET
                thing_score = a.thing_score + diff.score FROM (
                    SELECT
                        creator_id, sum(score) AS score FROM thing_diff GROUP BY creator_id) AS diff
                WHERE
                    a.person_id = diff.creator_id
                    AND diff.score != 0;
                RETURN NULL;
            END;
    $$);
    $b$,
    'thing',
    table_name);
END;
$a$;

CALL r.post_or_comment ('post');

CALL r.post_or_comment ('comment');

-- Create triggers that update counts in parent aggregates
CREATE FUNCTION r.parent_comment_ids (path ltree)
    RETURNS SETOF int
    LANGUAGE sql
    IMMUTABLE parallel safe
BEGIN
    ATOMIC
    SELECT
        comment_id::int
    FROM
        string_to_table (ltree2text (path), '.') AS comment_id
    -- Skip first and last
LIMIT (nlevel (path) - 2) OFFSET 1;

END;

CALL r.create_triggers ('comment', $$
BEGIN
    UPDATE
        person_aggregates AS a
    SET
        comment_count = a.comment_count + diff.comment_count
    FROM (
        SELECT
            (comment).creator_id, coalesce(sum(count_diff), 0) AS comment_count
        FROM select_old_and_new_rows AS old_and_new_rows
        WHERE
            r.is_counted (comment)
        GROUP BY (comment).creator_id) AS diff
WHERE
    a.person_id = diff.creator_id
        AND diff.comment_count != 0;

UPDATE
    comment_aggregates AS a
SET
    child_count = a.child_count + diff.child_count
FROM (
    SELECT
        parent_id,
        coalesce(sum(count_diff), 0) AS child_count
    FROM (
        -- For each inserted or deleted comment, this outputs 1 row for each parent comment.
        -- For example, this:
        --
        --  count_diff | (comment).path
        -- ------------+----------------
        --  1          | 0.5.6.7
        --  1          | 0.5.6.7.8
        --
        -- becomes this:
        --
        --  count_diff | parent_id
        -- ------------+-----------
        --  1          | 5
        --  1          | 6
        --  1          | 5
        --  1          | 6
        --  1          | 7
        SELECT
            count_diff,
            parent_id
        FROM
            select_old_and_new_rows AS old_and_new_rows,
            LATERAL r.parent_comment_ids ((comment).path) AS parent_id) AS expanded_old_and_new_rows
    GROUP BY
        parent_id) AS diff
WHERE
    a.comment_id = diff.parent_id
    AND diff.child_count != 0;

WITH post_diff AS (
    UPDATE
        post_aggregates AS a
    SET
        comments = a.comments + diff.comments,
        newest_comment_time = GREATEST (a.newest_comment_time, diff.newest_comment_time),
        newest_comment_time_necro = GREATEST (a.newest_comment_time_necro, diff.newest_comment_time_necro)
    FROM (
        SELECT
            post.id AS post_id,
            coalesce(sum(count_diff), 0) AS comments,
            -- Old rows are excluded using `count_diff = 1`
            max((comment).published) FILTER (WHERE count_diff = 1) AS newest_comment_time,
            max((comment).published) FILTER (WHERE count_diff = 1
                -- Ignore comments from the post's creator
                AND post.creator_id != (comment).creator_id
            -- Ignore comments on old posts
            AND post.published > ((comment).published - '2 days'::interval)) AS newest_comment_time_necro,
        r.is_counted (post.*) AS include_in_community_aggregates
    FROM
        select_old_and_new_rows AS old_and_new_rows
        LEFT JOIN post ON post.id = (comment).post_id
    WHERE
        r.is_counted (comment)
    GROUP BY
        post.id) AS diff
    WHERE
        a.post_id = diff.post_id
        AND (diff.comments,
            GREATEST (a.newest_comment_time, diff.newest_comment_time),
            GREATEST (a.newest_comment_time_necro, diff.newest_comment_time_necro)) != (0,
            a.newest_comment_time,
            a.newest_comment_time_necro)
    RETURNING
        a.community_id,
        diff.comments,
        diff.include_in_community_aggregates)
UPDATE
    community_aggregates AS a
SET
    comments = a.comments + diff.comments
FROM (
    SELECT
        community_id,
        sum(comments) AS comments
    FROM
        post_diff
    WHERE
        post_diff.include_in_community_aggregates
    GROUP BY
        community_id) AS diff
WHERE
    a.community_id = diff.community_id
    AND diff.comments != 0;

UPDATE
    site_aggregates AS a
SET
    comments = a.comments + diff.comments
FROM (
    SELECT
        coalesce(sum(count_diff), 0) AS comments
    FROM
        select_old_and_new_rows AS old_and_new_rows
    WHERE
        r.is_counted (comment)
        AND (comment).local) AS diff
WHERE
    diff.comments != 0;

RETURN NULL;

END;

$$);

CALL r.create_triggers ('post', $$
BEGIN
    UPDATE
        person_aggregates AS a
    SET
        post_count = a.post_count + diff.post_count
    FROM (
        SELECT
            (post).creator_id, coalesce(sum(count_diff), 0) AS post_count
        FROM select_old_and_new_rows AS old_and_new_rows
        WHERE
            r.is_counted (post)
        GROUP BY (post).creator_id) AS diff
WHERE
    a.person_id = diff.creator_id
        AND diff.post_count != 0;

UPDATE
    community_aggregates AS a
SET
    posts = a.posts + diff.posts
FROM (
    SELECT
        (post).community_id,
        coalesce(sum(count_diff), 0) AS posts
    FROM
        select_old_and_new_rows AS old_and_new_rows
    WHERE
        r.is_counted (post)
    GROUP BY
        (post).community_id) AS diff
WHERE
    a.community_id = diff.community_id
    AND diff.posts != 0;

UPDATE
    site_aggregates AS a
SET
    posts = a.posts + diff.posts
FROM (
    SELECT
        coalesce(sum(count_diff), 0) AS posts
    FROM
        select_old_and_new_rows AS old_and_new_rows
    WHERE
        r.is_counted (post)
        AND (post).local) AS diff
WHERE
    diff.posts != 0;

RETURN NULL;

END;

$$);

CALL r.create_triggers ('community', $$
BEGIN
    UPDATE
        site_aggregates AS a
    SET
        communities = a.communities + diff.communities
    FROM (
        SELECT
            coalesce(sum(count_diff), 0) AS communities
        FROM select_old_and_new_rows AS old_and_new_rows
        WHERE
            r.is_counted (community)
            AND (community).local) AS diff
WHERE
    diff.communities != 0;

RETURN NULL;

END;

$$);

CALL r.create_triggers ('person', $$
BEGIN
    UPDATE
        site_aggregates AS a
    SET
        users = a.users + diff.users
    FROM (
        SELECT
            coalesce(sum(count_diff), 0) AS users
        FROM select_old_and_new_rows AS old_and_new_rows
        WHERE (person).local) AS diff
WHERE
    diff.users != 0;

RETURN NULL;

END;

$$);

-- For community_aggregates.comments, don't include comments of deleted or removed posts
CREATE FUNCTION r.update_comment_count_from_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        community_aggregates AS a
    SET
        comments = a.comments + diff.comments
    FROM (
        SELECT
            old_post.community_id,
            sum((
                CASE WHEN r.is_counted (new_post.*) THEN
                    1
                ELSE
                    -1
                END) * post_aggregates.comments) AS comments
        FROM
            new_post
            INNER JOIN old_post ON new_post.id = old_post.id
                AND (r.is_counted (new_post.*) != r.is_counted (old_post.*))
                INNER JOIN post_aggregates ON post_aggregates.post_id = new_post.id
            GROUP BY
                old_post.community_id) AS diff
WHERE
    a.community_id = diff.community_id
        AND diff.comments != 0;
    RETURN NULL;
END;
$$;

CREATE TRIGGER comment_count
    AFTER UPDATE ON post REFERENCING OLD TABLE AS old_post NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.update_comment_count_from_post ();

-- Count subscribers for communities.
-- subscribers should be updated only when a local community is followed by a local or remote person.
-- subscribers_local should be updated only when a local person follows a local or remote community.
CALL r.create_triggers ('community_follower', $$
BEGIN
    UPDATE
        community_aggregates AS a
    SET
        subscribers = a.subscribers + diff.subscribers, subscribers_local = a.subscribers_local + diff.subscribers_local
    FROM (
        SELECT
            (community_follower).community_id, coalesce(sum(count_diff) FILTER (WHERE community.local), 0) AS subscribers, coalesce(sum(count_diff) FILTER (WHERE person.local), 0) AS subscribers_local
        FROM select_old_and_new_rows AS old_and_new_rows
    LEFT JOIN community ON community.id = (community_follower).community_id
    LEFT JOIN person ON person.id = (community_follower).person_id GROUP BY (community_follower).community_id) AS diff
WHERE
    a.community_id = diff.community_id
        AND (diff.subscribers, diff.subscribers_local) != (0, 0);

RETURN NULL;

END;

$$);

-- These triggers create and update rows in each aggregates table to match its associated table's rows.
-- Deleting rows and updating IDs are already handled by `CASCADE` in foreign key constraints.
CREATE FUNCTION r.comment_aggregates_from_comment ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO comment_aggregates (comment_id, published)
    SELECT
        id,
        published
    FROM
        new_comment;
    RETURN NULL;
END;
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON comment REFERENCING NEW TABLE AS new_comment
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.comment_aggregates_from_comment ();

CREATE FUNCTION r.community_aggregates_from_community ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO community_aggregates (community_id, published)
    SELECT
        id,
        published
    FROM
        new_community;
    RETURN NULL;
END;
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON community REFERENCING NEW TABLE AS new_community
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.community_aggregates_from_community ();

CREATE FUNCTION r.person_aggregates_from_person ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO person_aggregates (person_id)
    SELECT
        id
    FROM
        new_person;
    RETURN NULL;
END;
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON person REFERENCING NEW TABLE AS new_person
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.person_aggregates_from_person ();

CREATE FUNCTION r.post_aggregates_from_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO post_aggregates (post_id, published, newest_comment_time, newest_comment_time_necro, community_id, creator_id, instance_id, featured_community, featured_local)
    SELECT
        new_post.id,
        new_post.published,
        new_post.published,
        new_post.published,
        new_post.community_id,
        new_post.creator_id,
        community.instance_id,
        new_post.featured_community,
        new_post.featured_local
    FROM
        new_post
        INNER JOIN community ON community.id = new_post.community_id;
    RETURN NULL;
END;
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON post REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.post_aggregates_from_post ();

CREATE FUNCTION r.post_aggregates_from_post_update ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        post_aggregates
    SET
        featured_community = new_post.featured_community,
        featured_local = new_post.featured_local
    FROM
        new_post
        INNER JOIN old_post ON old_post.id = new_post.id
            AND (old_post.featured_community,
                old_post.featured_local) != (new_post.featured_community,
                new_post.featured_local)
    WHERE
        post_aggregates.post_id = new_post.id;
    RETURN NULL;
END;
$$;

CREATE TRIGGER aggregates_update
    AFTER UPDATE ON post REFERENCING OLD TABLE AS old_post NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.post_aggregates_from_post_update ();

CREATE FUNCTION r.site_aggregates_from_site ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- only 1 row can be in site_aggregates because of the index idx_site_aggregates_1_row_only.
    -- we only ever want to have a single value in site_aggregate because the site_aggregate triggers update all rows in that table.
    -- a cleaner check would be to insert it for the local_site but that would break assumptions at least in the tests
    INSERT INTO site_aggregates (site_id)
        VALUES (NEW.id)
    ON CONFLICT ((TRUE))
        DO NOTHING;
    RETURN NULL;
END;
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON site
    FOR EACH ROW
    EXECUTE FUNCTION r.site_aggregates_from_site ();

-- Change the order of some cascading deletions to make deletion triggers run before the deletion of rows that the triggers need to read
CREATE FUNCTION r.delete_comments_before_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM comment AS c
    WHERE c.post_id = OLD.id;
    RETURN OLD;
END;
$$;

CREATE TRIGGER delete_comments
    BEFORE DELETE ON post
    FOR EACH ROW
    EXECUTE FUNCTION r.delete_comments_before_post ();

CREATE FUNCTION r.delete_follow_before_person ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM community_follower AS c
    WHERE c.person_id = OLD.id;
    RETURN OLD;
END;
$$;

CREATE TRIGGER delete_follow
    BEFORE DELETE ON person
    FOR EACH ROW
    EXECUTE FUNCTION r.delete_follow_before_person ();

-- Triggers that change values before insert or update
CREATE FUNCTION r.comment_change_values ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
DECLARE
    id text = NEW.id::text;
BEGIN
    -- Make `path` end with `id` if it doesn't already
    IF NOT (NEW.path ~ ('*.' || id)::lquery) THEN
        NEW.path = NEW.path || id;
    END IF;
    -- Set local ap_id
    IF NEW.local THEN
        NEW.ap_id = coalesce(r.null_if_needs_replacement (NEW.ap_id), r.local_url('/comment/' || id));
    END IF;
    RETURN NEW;
END
$$;

CREATE TRIGGER change_values
    BEFORE INSERT OR UPDATE ON comment
    FOR EACH ROW
    EXECUTE FUNCTION r.comment_change_values ();

CREATE FUNCTION r.post_change_values ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Set local ap_id
    IF NEW.local THEN
        NEW.ap_id = coalesce(r.null_if_needs_replacement (NEW.ap_id), r.local_url('/post/' || NEW.id::text));
    END IF;
    RETURN NEW;
END
$$;

CREATE TRIGGER change_values
    BEFORE INSERT ON post
    FOR EACH ROW
    EXECUTE FUNCTION r.post_change_values ();

CREATE FUNCTION r.private_message_change_values ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Set local ap_id
    IF NEW.local THEN
        NEW.ap_id = coalesce(r.null_if_needs_replacement (NEW.ap_id), r.local_url('/private_message/' || NEW.id::text));
    END IF;
    RETURN NEW;
END
$$;

CREATE TRIGGER change_values
    BEFORE INSERT ON private_message
    FOR EACH ROW
    EXECUTE FUNCTION r.private_message_change_values ();

