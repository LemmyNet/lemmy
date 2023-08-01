-- Drop the old indexes
drop index idx_post_aggregates_featured_local_hot;
drop index idx_post_aggregates_featured_local_active;
drop index idx_post_aggregates_featured_local_score;
drop index idx_post_aggregates_featured_community_hot;
drop index idx_post_aggregates_featured_community_active;
drop index idx_post_aggregates_featured_community_score;
drop index idx_comment_aggregates_hot;
drop index idx_comment_aggregates_score;

-- Add a published desc, to the end of the hot and active ranks

-- Add missing most comments index
create index idx_post_aggregates_featured_local_most_comments on post_aggregates (featured_local desc, comments desc, published desc);
create index idx_post_aggregates_featured_community_most_comments on post_aggregates (featured_community desc, comments desc, published desc);

-- featured_local
create index idx_post_aggregates_featured_local_hot on post_aggregates (featured_local desc, hot_rank desc, published desc);
create index idx_post_aggregates_featured_local_active on post_aggregates (featured_local desc, hot_rank_active desc, published desc);
create index idx_post_aggregates_featured_local_score on post_aggregates (featured_local desc, score desc, published desc);

-- featured_community
create index idx_post_aggregates_featured_community_hot on post_aggregates (featured_community desc, hot_rank desc, published desc);
create index idx_post_aggregates_featured_community_active on post_aggregates (featured_community desc, hot_rank_active desc, published desc);
create index idx_post_aggregates_featured_community_score on post_aggregates (featured_community desc, score desc, published desc);

-- Fixing some comment aggregates ones
create index idx_comment_aggregates_hot on comment_aggregates (hot_rank desc, published desc);
create index idx_comment_aggregates_score on comment_aggregates (score desc, published desc);

