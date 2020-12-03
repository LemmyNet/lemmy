-- Add site aggregates
create table site_aggregates (
  id serial primary key,
  users bigint not null,
  posts bigint not null,
  comments bigint not null,
  communities bigint not null
);

insert into site_aggregates (users, posts, comments, communities)
  select ( select coalesce(count(*), 0) from user_) as users, 
  ( select coalesce(count(*), 0) from post) as posts,
  ( select coalesce(count(*), 0) from comment) as comments,
  ( select coalesce(count(*), 0) from community) as communities;

-- Add site aggregate triggers
-- user
create function site_aggregates_user()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update site_aggregates 
    set users = users + 1;
  ELSIF (TG_OP = 'DELETE') THEN
    update site_aggregates 
    set users = users - 1;
  END IF;
  return null;
end $$;

create trigger site_aggregates_user
after insert or delete on user_
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
    update site_aggregates 
    set posts = posts - 1;
  END IF;
  return null;
end $$;

create trigger site_aggregates_post
after insert or delete on post
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
    update site_aggregates 
    set comments = comments - 1;
  END IF;
  return null;
end $$;

create trigger site_aggregates_comment
after insert or delete on comment
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
    update site_aggregates 
    set communities = communities - 1;
  END IF;
  return null;
end $$;

create trigger site_aggregates_community
after insert or delete on community
execute procedure site_aggregates_community();

