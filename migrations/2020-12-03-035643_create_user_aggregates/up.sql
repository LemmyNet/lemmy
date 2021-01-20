-- Add user aggregates
create table user_aggregates (
  id serial primary key,
  user_id int references user_ on update cascade on delete cascade not null,
  post_count bigint not null default 0,
  post_score bigint not null default 0,
  comment_count bigint not null default 0,
  comment_score bigint not null default 0,
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


-- Add user aggregate triggers

-- initial user add
create function user_aggregates_user()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into user_aggregates (user_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from user_aggregates where user_id = OLD.id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_user
after insert or delete on user_
for each row
execute procedure user_aggregates_user();

-- post count
create function user_aggregates_post_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates 
    set post_count = post_count + 1 where user_id = NEW.creator_id;

  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates 
    set post_count = post_count - 1 where user_id = OLD.creator_id;

    -- If the post gets deleted, the score calculation trigger won't fire, 
    -- so you need to re-calculate
    update user_aggregates ua
    set post_score = pd.score
    from (
      select u.id,
      coalesce(0, sum(pl.score)) as score
      -- User join because posts could be empty
      from user_ u 
      left join post p on u.id = p.creator_id
      left join post_like pl on p.id = pl.post_id
      group by u.id
    ) pd 
    where ua.user_id = OLD.creator_id;

  END IF;
  return null;
end $$;

create trigger user_aggregates_post_count
after insert or delete on post
for each row
execute procedure user_aggregates_post_count();

-- post score
create function user_aggregates_post_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    -- Need to get the post creator, not the voter
    update user_aggregates ua
    set post_score = post_score + NEW.score
    from post p
    where ua.user_id = p.creator_id and p.id = NEW.post_id;
    
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates ua
    set post_score = post_score - OLD.score
    from post p
    where ua.user_id = p.creator_id and p.id = OLD.post_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_post_score
after insert or delete on post_like
for each row
execute procedure user_aggregates_post_score();

-- comment count
create function user_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates 
    set comment_count = comment_count + 1 where user_id = NEW.creator_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates 
    set comment_count = comment_count - 1 where user_id = OLD.creator_id;

    -- If the comment gets deleted, the score calculation trigger won't fire, 
    -- so you need to re-calculate
    update user_aggregates ua
    set comment_score = cd.score
    from (
      select u.id,
      coalesce(0, sum(cl.score)) as score
      -- User join because comments could be empty
      from user_ u 
      left join comment c on u.id = c.creator_id
      left join comment_like cl on c.id = cl.comment_id
      group by u.id
    ) cd 
    where ua.user_id = OLD.creator_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_comment_count
after insert or delete on comment
for each row
execute procedure user_aggregates_comment_count();

-- comment score
create function user_aggregates_comment_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    -- Need to get the post creator, not the voter
    update user_aggregates ua
    set comment_score = comment_score + NEW.score
    from comment c
    where ua.user_id = c.creator_id and c.id = NEW.comment_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates ua
    set comment_score = comment_score - OLD.score
    from comment c
    where ua.user_id = c.creator_id and c.id = OLD.comment_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_comment_score
after insert or delete on comment_like
for each row
execute procedure user_aggregates_comment_score();
