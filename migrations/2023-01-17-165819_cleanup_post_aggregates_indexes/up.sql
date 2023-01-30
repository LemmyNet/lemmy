-- Drop the old indexes
drop index idx_post_aggregates_newest_comment_time,
  idx_post_aggregates_comments,
  idx_post_aggregates_hot,
  idx_post_aggregates_active,
  idx_post_aggregates_score,
  idx_post_aggregates_published;

-- All of the post fetching queries now start with either 
-- featured_local desc, or featured_community desc, then the other sorts.
-- So you now need to double these indexes

create index idx_post_aggregates_featured_local_newest_comment_time on post_aggregates (featured_local desc, newest_comment_time desc);
create index idx_post_aggregates_featured_community_newest_comment_time on post_aggregates (featured_community desc, newest_comment_time desc);

create index idx_post_aggregates_featured_local_comments on post_aggregates (featured_local desc, comments desc);
create index idx_post_aggregates_featured_community_comments on post_aggregates (featured_community desc, comments desc);

create index idx_post_aggregates_featured_local_hot on post_aggregates (featured_local desc, hot_rank(score, published) desc, published desc);
create index idx_post_aggregates_featured_community_hot on post_aggregates (featured_community desc, hot_rank(score, published) desc, published desc);

create index idx_post_aggregates_featured_local_active on post_aggregates (featured_local desc, hot_rank(score, newest_comment_time) desc, newest_comment_time desc);
create index idx_post_aggregates_featured_community_active on post_aggregates (featured_community desc, hot_rank(score, newest_comment_time) desc, newest_comment_time desc);

create index idx_post_aggregates_featured_local_score on post_aggregates (featured_local desc, score desc);
create index idx_post_aggregates_featured_community_score on post_aggregates (featured_community desc, score desc);

create index idx_post_aggregates_featured_local_published on post_aggregates (featured_local desc, published desc);
create index idx_post_aggregates_featured_community_published on post_aggregates (featured_community desc, published desc);

