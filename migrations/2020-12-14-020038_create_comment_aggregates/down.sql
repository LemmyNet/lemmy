-- comment aggregates
DROP TABLE comment_aggregates;

DROP TRIGGER comment_aggregates_comment ON comment;

DROP TRIGGER comment_aggregates_score ON comment_like;

DROP FUNCTION comment_aggregates_comment, comment_aggregates_score;

