-- Your SQL goes here
CREATE INDEX idx_community_aggregates_nonzero_hotrank ON community_aggregates (published)
WHERE
    hot_rank != 0;

CREATE INDEX idx_comment_aggregates_nonzero_hotrank ON comment_aggregates (published)
WHERE
    hot_rank != 0;

CREATE INDEX idx_post_aggregates_nonzero_hotrank ON post_aggregates (published DESC)
WHERE
    hot_rank != 0 OR hot_rank_active != 0;

