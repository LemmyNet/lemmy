CREATE OR REPLACE FUNCTION comment_aggregates_comment ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO comment_aggregates (comment_id)
            VALUES (NEW.id);
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
        INSERT INTO post_aggregates (post_id)
            VALUES (NEW.id);
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
        INSERT INTO community_aggregates (community_id)
            VALUES (NEW.id);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM community_aggregates
        WHERE community_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

