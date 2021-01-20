-- Add community aggregates
create table community_aggregates (
  id serial primary key,
  community_id int references community on update cascade on delete cascade not null,
  subscribers bigint not null default 0,
  posts bigint not null default 0,
  comments bigint not null default 0,
  published timestamp not null default now(),
  unique (community_id)
);

insert into community_aggregates (community_id, subscribers, posts, comments, published)
  select 
    c.id,
    coalesce(cf.subs, 0) as subscribers,
    coalesce(cd.posts, 0) as posts,
    coalesce(cd.comments, 0) as comments,
    c.published
  from community c
  left join ( 
    select 
      p.community_id,
      count(distinct p.id) as posts,
      count(distinct ct.id) as comments
    from post p
    left join comment ct on p.id = ct.post_id
    group by p.community_id
  ) cd on cd.community_id = c.id
  left join ( 
    select 
      community_follower.community_id,
      count(*) as subs
    from community_follower
    group by community_follower.community_id
  ) cf on cf.community_id = c.id;

-- Add community aggregate triggers

-- initial community add
create function community_aggregates_community()
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

create trigger community_aggregates_community
after insert or delete on community
for each row
execute procedure community_aggregates_community();
-- post count
create function community_aggregates_post_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update community_aggregates 
    set posts = posts + 1 where community_id = NEW.community_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update community_aggregates 
    set posts = posts - 1 where community_id = OLD.community_id;

    -- Update the counts if the post got deleted
    update community_aggregates ca
    set posts = coalesce(cd.posts, 0),
    comments = coalesce(cd.comments, 0)
    from ( 
      select 
      c.id,
      count(distinct p.id) as posts,
      count(distinct ct.id) as comments
      from community c
      left join post p on c.id = p.community_id
      left join comment ct on p.id = ct.post_id
      group by c.id
    ) cd 
    where ca.community_id = OLD.community_id;
  END IF;
  return null;
end $$;

create trigger community_aggregates_post_count
after insert or delete on post
for each row
execute procedure community_aggregates_post_count();

-- comment count
create function community_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update community_aggregates ca
    set comments = comments + 1 from comment c, post p
    where p.id = c.post_id 
    and p.id = NEW.post_id 
    and ca.community_id = p.community_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update community_aggregates ca
    set comments = comments - 1 from comment c, post p
    where p.id = c.post_id 
    and p.id = OLD.post_id 
    and ca.community_id = p.community_id;

  END IF;
  return null;
end $$;

create trigger community_aggregates_comment_count
after insert or delete on comment
for each row
execute procedure community_aggregates_comment_count();

-- subscriber count
create function community_aggregates_subscriber_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update community_aggregates 
    set subscribers = subscribers + 1 where community_id = NEW.community_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update community_aggregates 
    set subscribers = subscribers - 1 where community_id = OLD.community_id;
  END IF;
  return null;
end $$;

create trigger community_aggregates_subscriber_count
after insert or delete on community_follower
for each row
execute procedure community_aggregates_subscriber_count();

