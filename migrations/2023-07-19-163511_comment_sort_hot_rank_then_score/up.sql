-- Alter the comment_aggregates hot sort to sort by score after hot_rank.
-- Reason being, is that hot_ranks go to zero after a few days, 
-- and then comments should be sorted by score, not published.

drop index idx_comment_aggregates_hot, idx_comment_aggregates_score;

create index idx_comment_aggregates_hot on comment_aggregates (hot_rank desc, score desc);

-- Remove published from this sort, its pointless
create index idx_comment_aggregates_score on comment_aggregates (score desc);
