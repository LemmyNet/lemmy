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


-- Add user aggregate triggers
-- post count
create function user_aggregates_post_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates 
    set post_count = post_count + 1 where user_id = NEW.user_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates 
    set post_count = post_count - 1 where user_id = OLD.user_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_post_count
after insert or delete on post
execute procedure user_aggregates_post_count();

-- post score
create function user_aggregates_post_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates 
    set post_score = post_score + NEW.score where user_id = NEW.user_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates 
    set post_score = post_score - OLD.score where user_id = OLD.user_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_post_score
after insert or delete on post_like
execute procedure user_aggregates_post_score();

-- comment count
create function user_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates 
    set comment_count = comment_count + 1 where user_id = NEW.user_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates 
    set comment_count = comment_count - 1 where user_id = OLD.user_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_comment_count
after insert or delete on comment
execute procedure user_aggregates_comment_count();

-- comment score
create function user_aggregates_comment_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update user_aggregates 
    set comment_score = comment_score + NEW.score where user_id = NEW.user_id;
  ELSIF (TG_OP = 'DELETE') THEN
    update user_aggregates 
    set comment_score = comment_score - OLD.score where user_id = OLD.user_id;
  END IF;
  return null;
end $$;

create trigger user_aggregates_comment_score
after insert or delete on comment_like
execute procedure user_aggregates_comment_score();
