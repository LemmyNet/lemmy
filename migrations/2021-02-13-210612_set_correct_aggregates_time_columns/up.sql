-- The published and updated columns on the aggregates tables are using now(), 
-- when they should use the correct published or updated columns
-- This is mainly a problem with federated posts being fetched

create or replace function comment_aggregates_comment()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into comment_aggregates (comment_id, published) values (NEW.id, NEW.published);
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
    insert into post_aggregates (post_id, published, newest_comment_time, newest_comment_time_necro) values (NEW.id, NEW.published, NEW.published, NEW.published);
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
    insert into community_aggregates (community_id, published) values (NEW.id, NEW.published);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from community_aggregates where community_id = OLD.id;
  END IF;
  return null;
end $$;
