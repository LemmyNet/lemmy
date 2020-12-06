-- Add community aggregates
create table community_aggregates (
  id serial primary key,
  community_id int references community on update cascade on delete cascade not null,
  subscribers bigint not null,
  posts bigint not null,
  comments bigint not null,
  unique (community_id)
);

insert into community_aggregates (community_id, subscribers, posts, comments)
  select 
    c.id,
    coalesce(cf.subs, 0::bigint) as subscribers,
    coalesce(cd.posts, 0::bigint) as posts,
    coalesce(cd.comments, 0::bigint) as comments
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
  END IF;
  return null;
end $$;

create trigger community_aggregates_post_count
after insert or delete on post
execute procedure community_aggregates_post_count();

-- comment count
create function community_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update community_aggregates 
    set comments = comments + 1 from comment c join post p on p.id = c.post_id and p.id = NEW.post_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update community_aggregates 
    set comments = comments - 1 from comment c join post p on p.id = c.post_id and p.id = OLD.post_id;
  END IF;
  return null;
end $$;

create trigger community_aggregates_comment_count
after insert or delete on comment
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
execute procedure community_aggregates_subscriber_count();

