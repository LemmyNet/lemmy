DROP TRIGGER IF EXISTS community_aggregates_post_count ON post;

DROP TRIGGER IF EXISTS community_aggregates_comment_count ON comment;

DROP TRIGGER IF EXISTS site_aggregates_comment_insert ON comment;

DROP TRIGGER IF EXISTS site_aggregates_comment_delete ON comment;

DROP TRIGGER IF EXISTS site_aggregates_post_insert ON post;

DROP TRIGGER IF EXISTS site_aggregates_post_delete ON post;

DROP TRIGGER IF EXISTS site_aggregates_community_insert ON community;

DROP TRIGGER IF EXISTS site_aggregates_community_delete ON community;

DROP TRIGGER IF EXISTS person_aggregates_post_count ON post;

DROP TRIGGER IF EXISTS person_aggregates_comment_count ON comment;

DROP FUNCTION was_removed_or_deleted (TG_OP text, OLD record, NEW record);

DROP FUNCTION was_restored_or_created (TG_OP text, OLD record, NEW record);

-- Community aggregate functions
CREATE OR REPLACE FUNCTION community_aggregates_post_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            community_aggregates
        SET
            posts = posts + 1
        WHERE
            community_id = NEW.community_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            community_aggregates
        SET
            posts = posts - 1
        WHERE
            community_id = OLD.community_id;
        -- Update the counts if the post got deleted
        UPDATE
            community_aggregates ca
        SET
            posts = coalesce(cd.posts, 0),
            comments = coalesce(cd.comments, 0)
        FROM (
            SELECT
                c.id,
                count(DISTINCT p.id) AS posts,
                count(DISTINCT ct.id) AS comments
            FROM
                community c
            LEFT JOIN post p ON c.id = p.community_id
            LEFT JOIN comment ct ON p.id = ct.post_id
        GROUP BY
            c.id) cd
    WHERE
        ca.community_id = OLD.community_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE OR REPLACE FUNCTION community_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            community_aggregates ca
        SET
            comments = comments + 1
        FROM
            comment c,
            post p
        WHERE
            p.id = c.post_id
            AND p.id = NEW.post_id
            AND ca.community_id = p.community_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            community_aggregates ca
        SET
            comments = comments - 1
        FROM
            comment c,
            post p
        WHERE
            p.id = c.post_id
            AND p.id = OLD.post_id
            AND ca.community_id = p.community_id;
    END IF;
    RETURN NULL;
END
$$;

-- Community aggregate triggers
CREATE TRIGGER community_aggregates_post_count
    AFTER INSERT OR DELETE ON post
    FOR EACH ROW
    EXECUTE PROCEDURE community_aggregates_post_count ();

CREATE TRIGGER community_aggregates_comment_count
    AFTER INSERT OR DELETE ON comment
    FOR EACH ROW
    EXECUTE PROCEDURE community_aggregates_comment_count ();

-- Site aggregate functions
CREATE OR REPLACE FUNCTION site_aggregates_post_insert ()
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

CREATE OR REPLACE FUNCTION site_aggregates_post_delete ()
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

CREATE OR REPLACE FUNCTION site_aggregates_comment_insert ()
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

CREATE OR REPLACE FUNCTION site_aggregates_comment_delete ()
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

CREATE OR REPLACE FUNCTION site_aggregates_community_insert ()
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

CREATE OR REPLACE FUNCTION site_aggregates_community_delete ()
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

-- Site update triggers
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

-- Person aggregate functions
CREATE OR REPLACE FUNCTION person_aggregates_post_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            person_aggregates
        SET
            post_count = post_count + 1
        WHERE
            person_id = NEW.creator_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            person_aggregates
        SET
            post_count = post_count - 1
        WHERE
            person_id = OLD.creator_id;
        -- If the post gets deleted, the score calculation trigger won't fire,
        -- so you need to re-calculate
        UPDATE
            person_aggregates ua
        SET
            post_score = pd.score
        FROM (
            SELECT
                u.id,
                coalesce(0, sum(pl.score)) AS score
                -- User join because posts could be empty
            FROM
                person u
            LEFT JOIN post p ON u.id = p.creator_id
            LEFT JOIN post_like pl ON p.id = pl.post_id
        GROUP BY
            u.id) pd
    WHERE
        ua.person_id = OLD.creator_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE OR REPLACE FUNCTION person_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            person_aggregates
        SET
            comment_count = comment_count + 1
        WHERE
            person_id = NEW.creator_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            person_aggregates
        SET
            comment_count = comment_count - 1
        WHERE
            person_id = OLD.creator_id;
        -- If the comment gets deleted, the score calculation trigger won't fire,
        -- so you need to re-calculate
        UPDATE
            person_aggregates ua
        SET
            comment_score = cd.score
        FROM (
            SELECT
                u.id,
                coalesce(0, sum(cl.score)) AS score
                -- User join because comments could be empty
            FROM
                person u
            LEFT JOIN comment c ON u.id = c.creator_id
            LEFT JOIN comment_like cl ON c.id = cl.comment_id
        GROUP BY
            u.id) cd
    WHERE
        ua.person_id = OLD.creator_id;
    END IF;
    RETURN NULL;
END
$$;

-- Person aggregate triggers
CREATE TRIGGER person_aggregates_post_count
    AFTER INSERT OR DELETE ON post
    FOR EACH ROW
    EXECUTE PROCEDURE person_aggregates_post_count ();

CREATE TRIGGER person_aggregates_comment_count
    AFTER INSERT OR DELETE ON comment
    FOR EACH ROW
    EXECUTE PROCEDURE person_aggregates_comment_count ();

