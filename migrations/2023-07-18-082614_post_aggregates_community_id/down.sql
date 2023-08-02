-- This file should undo anything in `up.sql`
CREATE OR REPLACE FUNCTION post_aggregates_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO post_aggregates (post_id, published, newest_comment_time, newest_comment_time_necro)
            VALUES (NEW.id, NEW.published, NEW.published, NEW.published);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM post_aggregates
        WHERE post_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

ALTER TABLE post_aggregates
    DROP COLUMN community_id,
    DROP COLUMN creator_id;

