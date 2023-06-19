-- Need to add immutable to the controversy_rank function in order to index by it

-- Controversy Rank:
--      if downvotes <= 0 or upvotes <= 0:
--          0 
--      else:
--          (upvotes + downvotes) * min(upvotes, downvotes) / max(upvotes, downvotes) 
create or replace function controversy_rank(upvotes numeric, downvotes numeric, score numeric)
returns integer as $$
begin
    if downvotes <= 0 or upvotes <= 0 then
        return 0;
    else
        return floor((upvotes + downvotes) * case when upvotes > downvotes then downvotes::float / upvotes::float else upvotes::float / downvotes::float end);
    end if;
end; $$
LANGUAGE plpgsql
IMMUTABLE;

-- Aggregates
alter table post_aggregates add column controversy_rank integer not null default 0;
alter table comment_aggregates add column controversy_rank integer not null default 0;

-- Populate them initially
-- Note: After initial population, these are updated in a periodic scheduled job, 
-- with only the last week being updated.
update post_aggregates set controversy_rank = controversy_rank(upvotes::numeric, downvotes::numeric, score::numeric);
update comment_aggregates set controversy_rank = controversy_rank(upvotes::numeric, downvotes::numeric, score::numeric);

-- Create single column indexes
create index idx_post_aggregates_controversy on post_aggregates (controversy_rank desc);
create index idx_comment_aggregates_controversy on comment_aggregates (controversy_rank desc);

