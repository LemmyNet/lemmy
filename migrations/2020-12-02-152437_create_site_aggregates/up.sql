-- Add site aggregates
CREATE TABLE site_aggregates (
    id serial PRIMARY KEY,
    site_id int REFERENCES site ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    users bigint NOT NULL DEFAULT 1,
    posts bigint NOT NULL DEFAULT 0,
    comments bigint NOT NULL DEFAULT 0,
    communities bigint NOT NULL DEFAULT 0
);

INSERT INTO site_aggregates (site_id, users, posts, comments, communities)
SELECT
    id AS site_id,
    (
        SELECT
            coalesce(count(*), 0)
        FROM
            user_
        WHERE
            local = TRUE) AS users,
    (
        SELECT
            coalesce(count(*), 0)
        FROM
            post
        WHERE
            local = TRUE) AS posts,
    (
        SELECT
            coalesce(count(*), 0)
        FROM
            comment
        WHERE
            local = TRUE) AS comments,
    (
        SELECT
            coalesce(count(*), 0)
        FROM
            community
        WHERE
            local = TRUE) AS communities
FROM
    site;

-- initial site add
CREATE FUNCTION site_aggregates_site ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO site_aggregates (site_id)
            VALUES (NEW.id);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM site_aggregates
        WHERE site_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER site_aggregates_site
    AFTER INSERT OR DELETE ON site
    FOR EACH ROW
    EXECUTE PROCEDURE site_aggregates_site ();

-- Add site aggregate triggers
-- user
CREATE FUNCTION site_aggregates_user_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        site_aggregates
    SET
        users = users + 1;
    RETURN NULL;
END
$$;

CREATE FUNCTION site_aggregates_user_delete ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Join to site since the creator might not be there anymore
    UPDATE
        site_aggregates sa
    SET
        users = users - 1
    FROM
        site s
    WHERE
        sa.site_id = s.id;
    RETURN NULL;
END
$$;

CREATE TRIGGER site_aggregates_user_insert
    AFTER INSERT ON user_
    FOR EACH ROW
    WHEN (NEW.local = TRUE)
    EXECUTE PROCEDURE site_aggregates_user_insert ();

CREATE TRIGGER site_aggregates_user_delete
    AFTER DELETE ON user_
    FOR EACH ROW
    WHEN (OLD.local = TRUE)
    EXECUTE PROCEDURE site_aggregates_user_delete ();

-- post
CREATE FUNCTION site_aggregates_post_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        site_aggregates
    SET
        posts = posts + 1;
    RETURN NULL;
END
$$;

CREATE FUNCTION site_aggregates_post_delete ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        site_aggregates sa
    SET
        posts = posts - 1
    FROM
        site s
    WHERE
        sa.site_id = s.id;
    RETURN NULL;
END
$$;

CREATE TRIGGER site_aggregates_post_insert
    AFTER INSERT ON post
    FOR EACH ROW
    WHEN (NEW.local = TRUE)
    EXECUTE PROCEDURE site_aggregates_post_insert ();

CREATE TRIGGER site_aggregates_post_delete
    AFTER DELETE ON post
    FOR EACH ROW
    WHEN (OLD.local = TRUE)
    EXECUTE PROCEDURE site_aggregates_post_delete ();

-- comment
CREATE FUNCTION site_aggregates_comment_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        site_aggregates
    SET
        comments = comments + 1;
    RETURN NULL;
END
$$;

CREATE FUNCTION site_aggregates_comment_delete ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        site_aggregates sa
    SET
        comments = comments - 1
    FROM
        site s
    WHERE
        sa.site_id = s.id;
    RETURN NULL;
END
$$;

CREATE TRIGGER site_aggregates_comment_insert
    AFTER INSERT ON comment
    FOR EACH ROW
    WHEN (NEW.local = TRUE)
    EXECUTE PROCEDURE site_aggregates_comment_insert ();

CREATE TRIGGER site_aggregates_comment_delete
    AFTER DELETE ON comment
    FOR EACH ROW
    WHEN (OLD.local = TRUE)
    EXECUTE PROCEDURE site_aggregates_comment_delete ();

-- community
CREATE FUNCTION site_aggregates_community_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        site_aggregates
    SET
        communities = communities + 1;
    RETURN NULL;
END
$$;

CREATE FUNCTION site_aggregates_community_delete ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        site_aggregates sa
    SET
        communities = communities - 1
    FROM
        site s
    WHERE
        sa.site_id = s.id;
    RETURN NULL;
END
$$;

CREATE TRIGGER site_aggregates_community_insert
    AFTER INSERT ON community
    FOR EACH ROW
    WHEN (NEW.local = TRUE)
    EXECUTE PROCEDURE site_aggregates_community_insert ();

CREATE TRIGGER site_aggregates_community_delete
    AFTER DELETE ON community
    FOR EACH ROW
    WHEN (OLD.local = TRUE)
    EXECUTE PROCEDURE site_aggregates_community_delete ();

