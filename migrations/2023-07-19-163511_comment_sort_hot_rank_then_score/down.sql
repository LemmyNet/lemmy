drop index idx_comment_aggregates_hot, idx_comment_aggregates_score;

create index idx_comment_aggregates_hot on comment_aggregates (hot_rank desc, published desc);
create index idx_comment_aggregates_score on comment_aggregates (score desc, published desc);
