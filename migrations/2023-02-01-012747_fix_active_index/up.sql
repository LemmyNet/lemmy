-- This should use the newest_comment_time_necro, not the newest_comment_time for the hot_rank
drop index 
  idx_post_aggregates_featured_local_active,
  idx_post_aggregates_featured_community_active;

create index idx_post_aggregates_featured_local_active on post_aggregates (featured_local desc, hot_rank(score, newest_comment_time_necro) desc, newest_comment_time_necro desc);
create index idx_post_aggregates_featured_community_active on post_aggregates (featured_community desc, hot_rank(score, newest_comment_time_necro) desc, newest_comment_time_necro desc);
