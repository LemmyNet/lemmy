-- Add indexes on published column (needed for hot_rank updates)

create index idx_community_aggregates_published on community_aggregates (published desc);
create index idx_comment_aggregates_published on comment_aggregates (published desc);