-- explain of Diesel generated SELECT queries shows improvement with this index
-- used both when listing posts for a specific community and in cross-reference for block lists
CREATE INDEX idx_post_aggregates_community ON post_aggregates (community_id DESC);
