drop trigger post_aggregates_comment_set_deleted on comment;
drop function post_aggregates_comment_deleted;

create or replace function post_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update post_aggregates pa
    set comments = comments + 1,
    newest_comment_time = NEW.published
    where pa.post_id = NEW.post_id;

    -- A 2 day necro-bump limit
    update post_aggregates pa
    set newest_comment_time_necro = NEW.published
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
