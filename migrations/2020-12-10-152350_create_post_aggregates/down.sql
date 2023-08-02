-- post aggregates
DROP TABLE post_aggregates;

DROP TRIGGER post_aggregates_post ON post;

DROP TRIGGER post_aggregates_comment_count ON comment;

DROP TRIGGER post_aggregates_score ON post_like;

DROP TRIGGER post_aggregates_stickied ON post;

DROP FUNCTION post_aggregates_post, post_aggregates_comment_count, post_aggregates_score, post_aggregates_stickied;

