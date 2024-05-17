DROP TRIGGER IF EXISTS post_aggregates_featured_local ON post;

DROP TRIGGER IF EXISTS post_aggregates_featured_community ON post;

DROP FUNCTION post_aggregates_featured_community;

DROP FUNCTION post_aggregates_featured_local;

ALTER TABLE post
    ADD stickied boolean NOT NULL DEFAULT FALSE;

UPDATE
    post
SET
    stickied = featured_community;

ALTER TABLE post
    DROP COLUMN featured_community;

ALTER TABLE post
    DROP COLUMN featured_local;

ALTER TABLE post_aggregates
    ADD stickied boolean NOT NULL DEFAULT FALSE;

UPDATE
    post_aggregates
SET
    stickied = featured_community;

ALTER TABLE post_aggregates
    DROP COLUMN featured_community;

ALTER TABLE post_aggregates
    DROP COLUMN featured_local;

ALTER TABLE mod_feature_post RENAME COLUMN featured TO stickied;

ALTER TABLE mod_feature_post
    DROP COLUMN is_featured_community;

ALTER TABLE mod_feature_post
    ALTER COLUMN stickied DROP NOT NULL;

ALTER TABLE mod_feature_post RENAME TO mod_sticky_post;

CREATE FUNCTION post_aggregates_stickied ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        post_aggregates pa
    SET
        stickied = NEW.stickied
    WHERE
        pa.post_id = NEW.id;
    RETURN NULL;
END
$$;

CREATE TRIGGER post_aggregates_stickied
    AFTER UPDATE ON post
    FOR EACH ROW
    WHEN (OLD.stickied IS DISTINCT FROM NEW.stickied)
    EXECUTE PROCEDURE post_aggregates_stickied ();

CREATE INDEX idx_post_aggregates_stickied_newest_comment_time ON post_aggregates (stickied DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_stickied_comments ON post_aggregates (stickied DESC, comments DESC);

CREATE INDEX idx_post_aggregates_stickied_hot ON post_aggregates (stickied DESC, hot_rank (score, published) DESC, published DESC);

CREATE INDEX idx_post_aggregates_stickied_active ON post_aggregates (stickied DESC, hot_rank (score, newest_comment_time_necro) DESC, newest_comment_time_necro DESC);

CREATE INDEX idx_post_aggregates_stickied_score ON post_aggregates (stickied DESC, score DESC);

CREATE INDEX idx_post_aggregates_stickied_published ON post_aggregates (stickied DESC, published DESC);

