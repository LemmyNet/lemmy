-- Create an admin person index
create index if not exists idx_person_admin on person (admin);

-- Compound indexes, using featured_, then the other sorts, proved to be much faster
-- Drop the old indexes
drop index idx_post_aggregates_score;
drop index idx_post_aggregates_published;
drop index idx_post_aggregates_newest_comment_time;
drop index idx_post_aggregates_newest_comment_time_necro;
drop index idx_post_aggregates_featured_community;
drop index idx_post_aggregates_featured_local;
drop index idx_post_aggregates_hot;
drop index idx_post_aggregates_active;

-- featured_local
create index idx_post_aggregates_featured_local_score on post_aggregates (featured_local desc, score desc);
create index idx_post_aggregates_featured_local_newest_comment_time on post_aggregates (featured_local desc, newest_comment_time desc);
create index idx_post_aggregates_featured_local_newest_comment_time_necro on post_aggregates (featured_local desc, newest_comment_time_necro desc);
create index idx_post_aggregates_featured_local_hot on post_aggregates (featured_local desc, hot_rank desc);
create index idx_post_aggregates_featured_local_active on post_aggregates (featured_local desc, hot_rank_active desc);
create index idx_post_aggregates_featured_local_published on post_aggregates (featured_local desc, published desc);
create index idx_post_aggregates_published on post_aggregates (published desc);

-- featured_community
create index idx_post_aggregates_featured_community_score on post_aggregates (featured_community desc, score desc);
create index idx_post_aggregates_featured_community_newest_comment_time on post_aggregates (featured_community desc, newest_comment_time desc);
create index idx_post_aggregates_featured_community_newest_comment_time_necro on post_aggregates (featured_community desc, newest_comment_time_necro desc);
create index idx_post_aggregates_featured_community_hot on post_aggregates (featured_community desc, hot_rank desc);
create index idx_post_aggregates_featured_community_active on post_aggregates (featured_community desc, hot_rank_active desc);
create index idx_post_aggregates_featured_community_published on post_aggregates (featured_community desc, published desc);


