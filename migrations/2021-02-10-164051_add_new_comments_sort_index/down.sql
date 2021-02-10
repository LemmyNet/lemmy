drop index idx_post_aggregates_newest_comment_time,
idx_post_aggregates_stickied_newest_comment_time,
idx_post_aggregates_stickied_comments;

alter table post_aggregates drop column newest_comment_time;

alter table post_aggregates rename column newest_comment_time_necro to newest_comment_time;

create or replace function post_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update post_aggregates pa
    set comments = comments + 1
    where pa.post_id = NEW.post_id;

    -- A 2 day necro-bump limit
    update post_aggregates pa
    set newest_comment_time = NEW.published
    where pa.post_id = NEW.post_id
    and published > ('now'::timestamp - '2 days'::interval);
  ELSIF (TG_OP = 'DELETE') THEN
    -- Join to post because that post may not exist anymore
    update post_aggregates pa
    set comments = comments - 1
    from post p
    where pa.post_id = p.id
    and pa.post_id = OLD.post_id;
  END IF;
  return null;
end $$;

