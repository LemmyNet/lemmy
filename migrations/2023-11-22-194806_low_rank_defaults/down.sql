ALTER TABLE community_aggregates
    ALTER COLUMN hot_rank SET DEFAULT 0.1728;

ALTER TABLE comment_aggregates
    ALTER COLUMN hot_rank SET DEFAULT 0.1728;

ALTER TABLE post_aggregates
    ALTER COLUMN hot_rank SET DEFAULT 0.1728,
    ALTER COLUMN hot_rank_active SET DEFAULT 0.1728,
    ALTER COLUMN scaled_rank SET DEFAULT 0.3621;

