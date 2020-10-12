-- Forgot to add hot rank active to these two triggers

create or replace function refresh_post()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'DELETE') THEN
    delete from post_aggregates_fast where id = OLD.id;

    -- Update community number of posts
    update community_aggregates_fast set number_of_posts = number_of_posts - 1 where id = OLD.community_id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from post_aggregates_fast where id = OLD.id;
    insert into post_aggregates_fast select * from post_aggregates_view where id = NEW.id on conflict (id) do nothing;
  ELSIF (TG_OP = 'INSERT') THEN
    insert into post_aggregates_fast select * from post_aggregates_view where id = NEW.id;

    -- Update that users number of posts, post score
    delete from user_fast where id = NEW.creator_id;
    insert into user_fast select * from user_view where id = NEW.creator_id on conflict (id) do nothing;
  
    -- Update community number of posts
    update community_aggregates_fast set number_of_posts = number_of_posts + 1 where id = NEW.community_id;

    -- Update the hot rank on the post table
    -- TODO this might not correctly update it, using a 1 week interval
    update post_aggregates_fast as paf
    set 
      hot_rank = pav.hot_rank,
      hot_rank_active = pav.hot_rank_active
    from post_aggregates_view as pav
    where paf.id = pav.id  and (pav.published > ('now'::timestamp - '1 week'::interval));
  END IF;

  return null;
end $$;

create or replace function refresh_comment()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'DELETE') THEN
    delete from comment_aggregates_fast where id = OLD.id;

    -- Update community number of comments
    update community_aggregates_fast as caf
    set number_of_comments = number_of_comments - 1
    from post as p
    where caf.id = p.community_id and p.id = OLD.post_id;

  ELSIF (TG_OP = 'UPDATE') THEN
    delete from comment_aggregates_fast where id = OLD.id;
    insert into comment_aggregates_fast select * from comment_aggregates_view where id = NEW.id on conflict (id) do nothing;
  ELSIF (TG_OP = 'INSERT') THEN
    insert into comment_aggregates_fast select * from comment_aggregates_view where id = NEW.id;

    -- Update user view due to comment count
    update user_fast 
    set number_of_comments = number_of_comments + 1
    where id = NEW.creator_id;
    
    -- Update post view due to comment count, new comment activity time, but only on new posts
    -- TODO this could be done more efficiently
    delete from post_aggregates_fast where id = NEW.post_id;
    insert into post_aggregates_fast select * from post_aggregates_view where id = NEW.post_id on conflict (id) do nothing;

    -- Update the comment hot_ranks as of last week
    update comment_aggregates_fast as caf
    set 
      hot_rank = cav.hot_rank,
      hot_rank_active = cav.hot_rank_active
    from comment_aggregates_view as cav
    where caf.id = cav.id and (cav.published > ('now'::timestamp - '1 week'::interval));

    -- Update the post ranks
    update post_aggregates_fast as paf
    set 
      hot_rank = pav.hot_rank,
      hot_rank_active = pav.hot_rank_active
    from post_aggregates_view as pav
    where paf.id = pav.id  and (pav.published > ('now'::timestamp - '1 week'::interval));

    -- Force the hot rank active as zero on 2 day-older posts (necro-bump)
    update post_aggregates_fast as paf
    set hot_rank_active = 0
    where paf.id = NEW.post_id and (paf.published < ('now'::timestamp - '2 days'::interval));

    -- Update community number of comments
    update community_aggregates_fast as caf
    set number_of_comments = number_of_comments + 1 
    from post as p
    where caf.id = p.community_id and p.id = NEW.post_id;

  END IF;

  return null;
end $$;
