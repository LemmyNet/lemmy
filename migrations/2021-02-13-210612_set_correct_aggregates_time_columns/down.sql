create or replace function comment_aggregates_comment()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into comment_aggregates (comment_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from comment_aggregates where comment_id = OLD.id;
  END IF;
  return null;
end $$;

create or replace function post_aggregates_post()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into post_aggregates (post_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from post_aggregates where post_id = OLD.id;
  END IF;
  return null;
end $$;

create or replace function community_aggregates_community()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into community_aggregates (community_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from community_aggregates where community_id = OLD.id;
  END IF;
  return null;
end $$;
