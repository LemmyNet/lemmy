-- Add indexes on published column (needed for hot_rank updates)
CREATE INDEX idx_community_aggregates_published ON community_aggregates (published DESC);

CREATE INDEX idx_comment_aggregates_published ON comment_aggregates (published DESC);

