-- This converts the old hot_rank functions, to columns

-- Remove the old compound indexes
DROP INDEX idx_post_aggregates_featured_local_newest_comment_time;
DROP INDEX idx_post_aggregates_featured_community_newest_comment_time;
DROP INDEX idx_post_aggregates_featured_local_comments;
DROP INDEX idx_post_aggregates_featured_community_comments;
DROP INDEX idx_post_aggregates_featured_local_hot;
DROP INDEX idx_post_aggregates_featured_community_hot;
DROP INDEX idx_post_aggregates_featured_local_score;
DROP INDEX idx_post_aggregates_featured_community_score;
DROP INDEX idx_post_aggregates_featured_local_published;
DROP INDEX idx_post_aggregates_featured_community_published;
DROP INDEX idx_post_aggregates_featured_local_active;
DROP INDEX idx_post_aggregates_featured_community_active;

DROP INDEX idx_comment_aggregates_hot;

DROP INDEX idx_community_aggregates_hot;

-- Add the new hot rank columns for post and comment aggregates
-- Note: 1728 is the result of the hot_rank function, with a score of 1, posted now
-- hot_rank = 10000*log10(1 + 3)/Power(2, 1.8)
alter table post_aggregates add column hot_rank integer not null default 1728;
alter table post_aggregates add column hot_rank_active integer not null default 1728;

alter table comment_aggregates add column hot_rank integer not null default 1728;

alter table community_aggregates add column hot_rank integer not null default 1728;

-- Populate them initially
-- Note: After initial population, these are updated in a periodic scheduled job, 
-- with only the last week being updated.
update post_aggregates set hot_rank_active = hot_rank(score::numeric, newest_comment_time_necro);
update post_aggregates set hot_rank = hot_rank(score::numeric, published);
update comment_aggregates set hot_rank = hot_rank(score::numeric, published);
update community_aggregates set hot_rank = hot_rank(subscribers::numeric, published);

-- Create single column indexes
create index idx_post_aggregates_score on post_aggregates (score desc);
create index idx_post_aggregates_published on post_aggregates (published desc);
create index idx_post_aggregates_newest_comment_time on post_aggregates (newest_comment_time desc);
create index idx_post_aggregates_newest_comment_time_necro on post_aggregates (newest_comment_time_necro desc);
create index idx_post_aggregates_featured_community on post_aggregates (featured_community desc);
create index idx_post_aggregates_featured_local on post_aggregates (featured_local desc);
create index idx_post_aggregates_hot on post_aggregates (hot_rank desc);
create index idx_post_aggregates_active on post_aggregates (hot_rank_active desc);

create index idx_comment_aggregates_hot on comment_aggregates (hot_rank desc);

create index idx_community_aggregates_hot on community_aggregates (hot_rank desc);
