DROP INDEX idx_comment_aggregates_hot, idx_comment_aggregates_score;

CREATE INDEX idx_comment_aggregates_hot ON comment_aggregates (hot_rank DESC, published DESC);

CREATE INDEX idx_comment_aggregates_score ON comment_aggregates (score DESC, published DESC);

