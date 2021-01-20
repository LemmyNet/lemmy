-- Add site aggregates
create table site_aggregates (
  id serial primary key,
  site_id int references site on update cascade on delete cascade not null,
  users bigint not null default 1,
  posts bigint not null default 0,
  comments bigint not null default 0,
  communities bigint not null default 0
);

insert into site_aggregates (site_id, users, posts, comments, communities)
  select id as site_id,
  ( select coalesce(count(*), 0) from user_ where local = true) as users, 
  ( select coalesce(count(*), 0) from post where local = true) as posts,
  ( select coalesce(count(*), 0) from comment where local = true) as comments,
  ( select coalesce(count(*), 0) from community where local = true) as communities
  from site;

-- initial site add
create function site_aggregates_site()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into site_aggregates (site_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from site_aggregates where site_id = OLD.id;
  END IF;
  return null;
end $$;

create trigger site_aggregates_site
after insert or delete on site
for each row
execute procedure site_aggregates_site();

-- Add site aggregate triggers
-- user
create function site_aggregates_user_insert()
returns trigger language plpgsql
as $$
begin
  update site_aggregates 
  set users = users + 1;
  return null;
end $$;

create function site_aggregates_user_delete()
returns trigger language plpgsql
as $$
begin
  -- Join to site since the creator might not be there anymore
  update site_aggregates sa
  set users = users - 1
  from site s
  where sa.site_id = s.id;
  return null;
end $$;

create trigger site_aggregates_user_insert
after insert on user_
for each row
when (NEW.local = true)
execute procedure site_aggregates_user_insert();

create trigger site_aggregates_user_delete
after delete on user_
for each row
when (OLD.local = true)
execute procedure site_aggregates_user_delete();

-- post
create function site_aggregates_post_insert()
returns trigger language plpgsql
as $$
begin
  update site_aggregates 
  set posts = posts + 1;
  return null;
end $$;

create function site_aggregates_post_delete()
returns trigger language plpgsql
as $$
begin
  update site_aggregates sa
  set posts = posts - 1
  from site s
  where sa.site_id = s.id;
  return null;
end $$;

create trigger site_aggregates_post_insert
after insert on post
for each row
when (NEW.local = true)
execute procedure site_aggregates_post_insert();

create trigger site_aggregates_post_delete
after delete on post
for each row
when (OLD.local = true)
execute procedure site_aggregates_post_delete();

-- comment
create function site_aggregates_comment_insert()
returns trigger language plpgsql
as $$
begin
  update site_aggregates 
  set comments = comments + 1;
  return null;
end $$;

create function site_aggregates_comment_delete()
returns trigger language plpgsql
as $$
begin
  update site_aggregates sa
  set comments = comments - 1
  from site s
  where sa.site_id = s.id;
  return null;
end $$;

create trigger site_aggregates_comment_insert
after insert on comment
for each row
when (NEW.local = true)
execute procedure site_aggregates_comment_insert();

create trigger site_aggregates_comment_delete
after delete on comment
for each row
when (OLD.local = true)
execute procedure site_aggregates_comment_delete();

-- community
create function site_aggregates_community_insert()
returns trigger language plpgsql
as $$
begin
  update site_aggregates 
  set communities = communities + 1;
  return null;
end $$;

create function site_aggregates_community_delete()
returns trigger language plpgsql
as $$
begin
  update site_aggregates sa
  set communities = communities - 1
  from site s
  where sa.site_id = s.id;
  return null;
end $$;

create trigger site_aggregates_community_insert
after insert on community
for each row
when (NEW.local = true)
execute procedure site_aggregates_community_insert();

create trigger site_aggregates_community_delete
after delete on community
for each row
when (OLD.local = true)
execute procedure site_aggregates_community_delete();
