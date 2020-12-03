-- Add user aggregates
create table user_aggregates (
  id serial primary key,
  user_id int references user_ on update cascade on delete cascade not null,
  post_count bigint not null,
  post_score bigint not null,
  comment_count bigint not null,
  comment_score bigint not null,
  unique (user_id)
);

insert into user_aggregates (user_id, post_count, post_score, comment_count, comment_score)
  select u.id,
  coalesce(pd.posts, 0),
  coalesce(pd.score, 0),
  coalesce(cd.comments, 0),
  coalesce(cd.score, 0)
  from user_ u
  left join (
    select p.creator_id,
      count(distinct p.id) as posts,
      sum(pl.score) as score
      from post p
      left join post_like pl on p.id = pl.post_id
      group by p.creator_id
    ) pd on u.id = pd.creator_id
  left join ( 
    select c.creator_id,
    count(distinct c.id) as comments,
    sum(cl.score) as score
    from comment c
    left join comment_like cl on c.id = cl.comment_id
    group by c.creator_id
  ) cd on u.id = cd.creator_id;


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

