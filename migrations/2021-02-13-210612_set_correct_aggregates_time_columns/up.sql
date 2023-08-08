-- The published and updated columns on the aggregates tables are using now(),
-- when they should use the correct published or updated columns
-- This is mainly a problem with federated posts being fetched
CREATE OR REPLACE FUNCTION comment_aggregates_comment ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO comment_aggregates (comment_id, published)
            VALUES (NEW.id, NEW.published);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM comment_aggregates
        WHERE comment_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

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

CREATE OR REPLACE FUNCTION community_aggregates_community ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO community_aggregates (community_id, published)
            VALUES (NEW.id, NEW.published);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM community_aggregates
        WHERE community_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

