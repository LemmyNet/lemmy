-- Add post aggregates
create table post_aggregates (
  id serial primary key,
  post_id int references post on update cascade on delete cascade not null,
  comments bigint not null default 0,
  score bigint not null default 0,
  upvotes bigint not null default 0,
  downvotes bigint not null default 0,
  stickied boolean not null default false,
  published timestamp not null default now(),
  newest_comment_time timestamp not null default now(),
  unique (post_id)
);

insert into post_aggregates (post_id, comments, score, upvotes, downvotes, stickied, published, newest_comment_time)
  select 
    p.id,
    coalesce(ct.comments, 0::bigint) as comments,
    coalesce(pl.score, 0::bigint) as score,
    coalesce(pl.upvotes, 0::bigint) as upvotes,
    coalesce(pl.downvotes, 0::bigint) as downvotes,
    p.stickied,
    p.published,
    greatest(ct.recent_comment_time, p.published) as newest_activity_time
  from post p
  left join ( 
    select comment.post_id,
    count(*) as comments,
    max(comment.published) as recent_comment_time
    from comment
    group by comment.post_id
  ) ct on ct.post_id = p.id
  left join ( 
    select post_like.post_id,
    sum(post_like.score) as score,
    sum(post_like.score) filter (where post_like.score = 1) as upvotes,
    -sum(post_like.score) filter (where post_like.score = '-1'::integer) as downvotes
    from post_like
    group by post_like.post_id
  ) pl on pl.post_id = p.id;

-- Add community aggregate triggers

-- initial post add
create function post_aggregates_post()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into post_aggregates (post_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from post_aggregates where post_id = OLD.id;
  END IF;
  return null;
end $$;

create trigger post_aggregates_post
after insert or delete on post
for each row
execute procedure post_aggregates_post();

-- comment count
create function post_aggregates_comment_count()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update post_aggregates pa
    set comments = comments + 1
    where pa.post_id = NEW.post_id;

    -- A 2 day necro-bump limit
    update post_aggregates pa
    set newest_comment_time = NEW.published
    where pa.post_id = NEW.post_id
    and published > ('now'::timestamp - '2 days'::interval);
  ELSIF (TG_OP = 'DELETE') THEN
    -- Join to post because that post may not exist anymore
    update post_aggregates pa
    set comments = comments - 1
    from post p
    where pa.post_id = p.id
    and pa.post_id = OLD.post_id;
  END IF;
  return null;
end $$;

create trigger post_aggregates_comment_count
after insert or delete on comment
for each row
execute procedure post_aggregates_comment_count();

-- post score
create function post_aggregates_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update post_aggregates pa
    set score = score + NEW.score,
    upvotes = case when NEW.score = 1 then upvotes + 1 else upvotes end,
    downvotes = case when NEW.score = -1 then downvotes + 1 else downvotes end
    where pa.post_id = NEW.post_id;

  ELSIF (TG_OP = 'DELETE') THEN
    -- Join to post because that post may not exist anymore
    update post_aggregates pa
    set score = score - OLD.score,
    upvotes = case when OLD.score = 1 then upvotes - 1 else upvotes end,
    downvotes = case when OLD.score = -1 then downvotes - 1 else downvotes end
    from post p
    where pa.post_id = p.id
    and pa.post_id = OLD.post_id;

  END IF;
  return null;
end $$;

create trigger post_aggregates_score
after insert or delete on post_like
for each row
execute procedure post_aggregates_score();

-- post stickied
create function post_aggregates_stickied()
returns trigger language plpgsql
as $$
begin
  update post_aggregates pa
  set stickied = NEW.stickied
  where pa.post_id = NEW.id;

  return null;
end $$;

create trigger post_aggregates_stickied
after update on post
for each row
when (OLD.stickied is distinct from NEW.stickied)
execute procedure post_aggregates_stickied();
