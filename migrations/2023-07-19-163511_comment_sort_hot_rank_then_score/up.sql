-- Alter the comment_aggregates hot sort to sort by score after hot_rank.
-- Reason being, is that hot_ranks go to zero after a few days,
-- and then comments should be sorted by score, not published.
DROP INDEX idx_comment_aggregates_hot, idx_comment_aggregates_score;

CREATE INDEX idx_comment_aggregates_hot ON comment_aggregates (hot_rank DESC, score DESC);

-- Remove published from this sort, its pointless
CREATE INDEX idx_comment_aggregates_score ON comment_aggregates (score DESC);

