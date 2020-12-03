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
create function site_aggregates_user_increment()
returns trigger language plpgsql
as $$
begin
  update site_aggregates 
  set users = users + 1;
  return null;
end $$;

create trigger site_aggregates_insert_user
after insert on user_
execute procedure site_aggregates_user_increment();

create function site_aggregates_user_decrement()
returns trigger language plpgsql
as $$
begin
  update site_aggregates 
  set users = users - 1;
  return null;
end $$;

create trigger site_aggregates_delete_user
after delete on user_
execute procedure site_aggregates_user_decrement();

-- post
create function site_aggregates_post_increment()
returns trigger language plpgsql
as $$
begin
  update site_aggregates 
  set posts = posts + 1;
  return null;
end $$;

create trigger site_aggregates_insert_post
after insert on post
execute procedure site_aggregates_post_increment();

create function site_aggregates_post_decrement()
returns trigger language plpgsql
as $$
begin
  update site_aggregates 
  set posts = posts - 1;
  return null;
end $$;

create trigger site_aggregates_delete_post
after delete on post
execute procedure site_aggregates_post_decrement();

-- comment
create function site_aggregates_comment_increment()
returns trigger language plpgsql
as $$
begin
  update site_aggregates 
  set comments = comments + 1;
  return null;
end $$;

create trigger site_aggregates_insert_comment
after insert on comment
execute procedure site_aggregates_comment_increment();

create function site_aggregates_comment_decrement()
returns trigger language plpgsql
as $$
begin
  update site_aggregates 
  set comments = comments - 1;
  return null;
end $$;

create trigger site_aggregates_delete_comment
after delete on comment
execute procedure site_aggregates_comment_decrement();

-- community
create function site_aggregates_community_increment()
returns trigger language plpgsql
as $$
begin
  update site_aggregates 
  set communities = communities + 1;
  return null;
end $$;

create trigger site_aggregates_insert_community
after insert on community
execute procedure site_aggregates_community_increment();

create function site_aggregates_community_decrement()
returns trigger language plpgsql
as $$
begin
  update site_aggregates 
  set communities = communities - 1;
  return null;
end $$;

create trigger site_aggregates_delete_community
after delete on community
execute procedure site_aggregates_community_decrement();

