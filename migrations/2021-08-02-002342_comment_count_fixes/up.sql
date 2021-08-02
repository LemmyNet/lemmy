-- Creating a new trigger for when comment.deleted is updated

create or replace function post_aggregates_comment_deleted()
returns trigger language plpgsql
as $$
begin
  IF NEW.deleted = TRUE THEN
    update post_aggregates pa
    set comments = comments - 1
    where pa.post_id = NEW.post_id;
  ELSE 
    update post_aggregates pa
    set comments = comments + 1
    where pa.post_id = NEW.post_id;
  END IF;
  return null;
end $$;

create trigger post_aggregates_comment_set_deleted 
after update of deleted on comment
for each row
execute procedure post_aggregates_comment_deleted();

-- Fix issue with being able to necro-bump your own post
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
    from post p
    where pa.post_id = p.id
    and pa.post_id = NEW.post_id
    -- Fix issue with being able to necro-bump your own post
    and NEW.creator_id != p.creator_id
    and pa.published > ('now'::timestamp - '2 days'::interval);

  ELSIF (TG_OP = 'DELETE') THEN
    -- Join to post because that post may not exist anymore
    update post_aggregates pa
    set comments = comments - 1
    from post p
    where pa.post_id = p.id
    and pa.post_id = OLD.post_id;
  ELSIF (TG_OP = 'UPDATE') THEN
    -- Join to post because that post may not exist anymore
    update post_aggregates pa
    set comments = comments - 1
    from post p
    where pa.post_id = p.id
    and pa.post_id = OLD.post_id;
  END IF;
  return null;
end $$;
