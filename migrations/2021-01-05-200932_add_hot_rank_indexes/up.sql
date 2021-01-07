-- Need to add immutable to the hot_rank function in order to index by it

-- Rank = ScaleFactor * sign(Score) * log(1 + abs(Score)) / (Time + 2)^Gravity
create or replace function hot_rank(
  score numeric,
  published timestamp without time zone)
returns integer as $$
begin
  -- hours_diff:=EXTRACT(EPOCH FROM (timezone('utc',now()) - published))/3600
  return floor(10000*log(greatest(1,score+3)) / power(((EXTRACT(EPOCH FROM (timezone('utc',now()) - published))/3600) + 2), 1.8))::integer;
end; $$
LANGUAGE plpgsql
IMMUTABLE;

-- Post_aggregates
create index idx_post_aggregates_stickied_hot on post_aggregates (stickied desc, hot_rank(score, published) desc, published desc);
create index idx_post_aggregates_hot on post_aggregates (hot_rank(score, published) desc, published desc);

create index idx_post_aggregates_stickied_active on post_aggregates (stickied desc, hot_rank(score, newest_comment_time) desc, newest_comment_time desc);
create index idx_post_aggregates_active on post_aggregates (hot_rank(score, newest_comment_time) desc, newest_comment_time desc);

create index idx_post_aggregates_stickied_score on post_aggregates (stickied desc, score desc);
create index idx_post_aggregates_score on post_aggregates (score desc);

create index idx_post_aggregates_stickied_published on post_aggregates (stickied desc, published desc);
create index idx_post_aggregates_published on post_aggregates (published desc);

-- Comment
create index idx_comment_published on comment (published desc);

-- Comment_aggregates
create index idx_comment_aggregates_hot on comment_aggregates (hot_rank(score, published) desc, published desc);
create index idx_comment_aggregates_score on comment_aggregates (score desc);

-- User
create index idx_user_published on user_ (published desc);

-- User_aggregates
create index idx_user_aggregates_comment_score on user_aggregates (comment_score desc);

-- Community
create index idx_community_published on community (published desc);

-- Community_aggregates
create index idx_community_aggregates_hot on community_aggregates (hot_rank(subscribers, published) desc, published desc);
create index idx_community_aggregates_subscribers on community_aggregates (subscribers desc);



