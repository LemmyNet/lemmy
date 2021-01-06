-- Add comment aggregates
create table comment_aggregates (
  id serial primary key,
  comment_id int references comment on update cascade on delete cascade not null,
  score bigint not null default 0,
  upvotes bigint not null default 0,
  downvotes bigint not null default 0,
  published timestamp not null default now(),
  unique (comment_id)
);

insert into comment_aggregates (comment_id, score, upvotes, downvotes, published)
  select 
    c.id,
    COALESCE(cl.total, 0::bigint) AS score,
    COALESCE(cl.up, 0::bigint) AS upvotes,
    COALESCE(cl.down, 0::bigint) AS downvotes,
    c.published
  from comment c
  left join ( select l.comment_id as id,
    sum(l.score) as total,
    count(
      case
      when l.score = 1 then 1
      else null::integer
      end) as up,
    count(
      case
      when l.score = '-1'::integer then 1
      else null::integer
      end) as down
    from comment_like l
    group by l.comment_id) cl on cl.id = c.id;

-- Add comment aggregate triggers

-- initial comment add
create function comment_aggregates_comment()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    insert into comment_aggregates (comment_id) values (NEW.id);
  ELSIF (TG_OP = 'DELETE') THEN
    delete from comment_aggregates where comment_id = OLD.id;
  END IF;
  return null;
end $$;

create trigger comment_aggregates_comment
after insert or delete on comment
for each row
execute procedure comment_aggregates_comment();

-- comment score
create function comment_aggregates_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update comment_aggregates ca
    set score = score + NEW.score,
    upvotes = case when NEW.score = 1 then upvotes + 1 else upvotes end,
    downvotes = case when NEW.score = -1 then downvotes + 1 else downvotes end
    where ca.comment_id = NEW.comment_id;

  ELSIF (TG_OP = 'DELETE') THEN
    -- Join to comment because that comment may not exist anymore
    update comment_aggregates ca
    set score = score - OLD.score,
    upvotes = case when OLD.score = 1 then upvotes - 1 else upvotes end,
    downvotes = case when OLD.score = -1 then downvotes - 1 else downvotes end
    from comment c
    where ca.comment_id = c.id
    and ca.comment_id = OLD.comment_id;

  END IF;
  return null;
end $$;

create trigger comment_aggregates_score
after insert or delete on comment_like
for each row
execute procedure comment_aggregates_score();
