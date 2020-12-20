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
  ( select coalesce(count(*), 0) from user_) as users, 
  ( select coalesce(count(*), 0) from post) as posts,
  ( select coalesce(count(*), 0) from comment) as comments,
  ( select coalesce(count(*), 0) from community) as communities
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
create or replace function site_aggregates_user()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update site_aggregates 
    set users = users + 1;
  ELSIF (TG_OP = 'DELETE') THEN
    -- Join to site since the creator might not be there anymore
    update site_aggregates sa
    set users = users - 1
    from site s
    where sa.site_id = s.id;
  END IF;
  return null;
end $$;

create trigger site_aggregates_user
after insert or delete on user_
for each row
execute procedure site_aggregates_user();

-- post
create function site_aggregates_post()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update site_aggregates 
    set posts = posts + 1;
  ELSIF (TG_OP = 'DELETE') THEN
    update site_aggregates sa
    set posts = posts - 1
    from site s
    where sa.site_id = s.id;
  END IF;
  return null;
end $$;

create trigger site_aggregates_post
after insert or delete on post
for each row
execute procedure site_aggregates_post();

-- comment
create function site_aggregates_comment()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update site_aggregates 
    set comments = comments + 1;
  ELSIF (TG_OP = 'DELETE') THEN
    update site_aggregates sa
    set comments = comments - 1
    from site s
    where sa.site_id = s.id;
  END IF;
  return null;
end $$;

create trigger site_aggregates_comment
after insert or delete on comment
for each row
execute procedure site_aggregates_comment();

-- community
create function site_aggregates_community()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update site_aggregates 
    set communities = communities + 1;
  ELSIF (TG_OP = 'DELETE') THEN
    update site_aggregates sa
    set communities = communities - 1
    from site s
    where sa.site_id = s.id;
  END IF;
  return null;
end $$;

create trigger site_aggregates_community
after insert or delete on community
for each row
execute procedure site_aggregates_community();

