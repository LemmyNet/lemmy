-- User aggregates
DROP TABLE user_aggregates;

DROP TRIGGER user_aggregates_user ON user_;

DROP TRIGGER user_aggregates_post_count ON post;

DROP TRIGGER user_aggregates_post_score ON post_like;

DROP TRIGGER user_aggregates_comment_count ON comment;

DROP TRIGGER user_aggregates_comment_score ON comment_like;

DROP FUNCTION user_aggregates_user, user_aggregates_post_count, user_aggregates_post_score, user_aggregates_comment_count, user_aggregates_comment_score;

