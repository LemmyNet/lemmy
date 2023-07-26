-- Need to add immutable to the controversy_rank function in order to index by it

-- Controversy Rank:
--      if downvotes <= 0 or upvotes <= 0:
--          0 
--      else:
--          (upvotes + downvotes) * min(upvotes, downvotes) / max(upvotes, downvotes) 
create or replace function controversy_rank(upvotes numeric, downvotes numeric)
returns float as $$
begin
    if downvotes <= 0 or upvotes <= 0 then
        return 0;
    else
        return (upvotes + downvotes) *
            case when upvotes > downvotes
                then downvotes::float / upvotes::float
                else upvotes::float / downvotes::float
            end;
    end if;
end; $$
LANGUAGE plpgsql
IMMUTABLE;

-- Aggregates
alter table post_aggregates add column controversy_rank float not null default 0;
alter table comment_aggregates add column controversy_rank float not null default 0;

-- Populate them initially
-- Note: After initial population, these are updated with vote triggers
update post_aggregates set controversy_rank = controversy_rank(upvotes::numeric, downvotes::numeric);
update comment_aggregates set controversy_rank = controversy_rank(upvotes::numeric, downvotes::numeric);

-- Create single column indexes
create index idx_post_aggregates_featured_local_controversy on post_aggregates (featured_local desc, controversy_rank desc);
create index idx_post_aggregates_featured_community_controversy on post_aggregates (featured_community desc, controversy_rank desc);
create index idx_comment_aggregates_controversy on comment_aggregates (controversy_rank desc);

-- Update post_aggregates_score trigger function to include controversy_rank update
create or replace function post_aggregates_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update post_aggregates pa
    set score = score + NEW.score,
    upvotes = case when NEW.score = 1 then upvotes + 1 else upvotes end,
    downvotes = case when NEW.score = -1 then downvotes + 1 else downvotes end,
    controversy_rank = controversy_rank(pa.upvotes + case when NEW.score = 1 then 1 else 0 end::numeric, 
                                         pa.downvotes + case when NEW.score = -1 then 1 else 0 end::numeric)
    where pa.post_id = NEW.post_id;

  ELSIF (TG_OP = 'DELETE') THEN
    -- Join to post because that post may not exist anymore
    update post_aggregates pa
    set score = score - OLD.score,
    upvotes = case when OLD.score = 1 then upvotes - 1 else upvotes end,
    downvotes = case when OLD.score = -1 then downvotes - 1 else downvotes end,
    controversy_rank = controversy_rank(pa.upvotes + case when NEW.score = 1 then 1 else 0 end::numeric, 
                                         pa.downvotes + case when NEW.score = -1 then 1 else 0 end::numeric)
    from post p
    where pa.post_id = p.id
    and pa.post_id = OLD.post_id;

  END IF;
  return null;
end $$;

-- Update comment_aggregates_score trigger function to include controversy_rank update
create or replace function comment_aggregates_score()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'INSERT') THEN
    update comment_aggregates ca
    set score = score + NEW.score,
    upvotes = case when NEW.score = 1 then upvotes + 1 else upvotes end,
    downvotes = case when NEW.score = -1 then downvotes + 1 else downvotes end,
    controversy_rank = controversy_rank(ca.upvotes + case when NEW.score = 1 then 1 else 0 end::numeric, 
                                         ca.downvotes + case when NEW.score = -1 then 1 else 0 end::numeric)
    where ca.comment_id = NEW.comment_id;

  ELSIF (TG_OP = 'DELETE') THEN
    -- Join to comment because that comment may not exist anymore
    update comment_aggregates ca
    set score = score - OLD.score,
    upvotes = case when OLD.score = 1 then upvotes - 1 else upvotes end,
    downvotes = case when OLD.score = -1 then downvotes - 1 else downvotes end,
    controversy_rank = controversy_rank(ca.upvotes + case when NEW.score = 1 then 1 else 0 end::numeric, 
                                         ca.downvotes + case when NEW.score = -1 then 1 else 0 end::numeric)
    from comment c
    where ca.comment_id = c.id
    and ca.comment_id = OLD.comment_id;

  END IF;
  return null;
end $$;

