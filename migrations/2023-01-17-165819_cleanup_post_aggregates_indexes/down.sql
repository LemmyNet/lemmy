-- Drop the new indexes
drop index idx_post_aggregates_featured_local_newest_comment_time,
  idx_post_aggregates_featured_community_newest_comment_time,
  idx_post_aggregates_featured_local_comments,
  idx_post_aggregates_featured_community_comments,
  idx_post_aggregates_featured_local_hot,
  idx_post_aggregates_featured_community_hot,
  idx_post_aggregates_featured_local_active,
  idx_post_aggregates_featured_community_active,
  idx_post_aggregates_featured_local_score,
  idx_post_aggregates_featured_community_score,
  idx_post_aggregates_featured_local_published,
  idx_post_aggregates_featured_community_published;

-- Create the old indexes
create index idx_post_aggregates_newest_comment_time on post_aggregates (newest_comment_time desc);
create index idx_post_aggregates_comments on post_aggregates (comments desc);
create index idx_post_aggregates_hot on post_aggregates (hot_rank(score, published) desc, published desc);
create index idx_post_aggregates_active on post_aggregates (hot_rank(score, newest_comment_time) desc, newest_comment_time desc);
create index idx_post_aggregates_score on post_aggregates (score desc);
create index idx_post_aggregates_published on post_aggregates (published desc);

