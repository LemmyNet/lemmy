-- Add user aggregates
CREATE TABLE user_aggregates (
    id serial PRIMARY KEY,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_count bigint NOT NULL DEFAULT 0,
    post_score bigint NOT NULL DEFAULT 0,
    comment_count bigint NOT NULL DEFAULT 0,
    comment_score bigint NOT NULL DEFAULT 0,
    UNIQUE (user_id)
);

INSERT INTO user_aggregates (user_id, post_count, post_score, comment_count, comment_score)
SELECT
    u.id,
    coalesce(pd.posts, 0),
    coalesce(pd.score, 0),
    coalesce(cd.comments, 0),
    coalesce(cd.score, 0)
FROM
    user_ u
    LEFT JOIN (
        SELECT
            p.creator_id,
            count(DISTINCT p.id) AS posts,
            sum(pl.score) AS score
        FROM
            post p
            LEFT JOIN post_like pl ON p.id = pl.post_id
        GROUP BY
            p.creator_id) pd ON u.id = pd.creator_id
    LEFT JOIN (
        SELECT
            c.creator_id,
            count(DISTINCT c.id) AS comments,
            sum(cl.score) AS score
        FROM
            comment c
            LEFT JOIN comment_like cl ON c.id = cl.comment_id
        GROUP BY
            c.creator_id) cd ON u.id = cd.creator_id;

-- Add user aggregate triggers
-- initial user add
CREATE FUNCTION user_aggregates_user ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO user_aggregates (user_id)
            VALUES (NEW.id);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM user_aggregates
        WHERE user_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER user_aggregates_user
    AFTER INSERT OR DELETE ON user_
    FOR EACH ROW
    EXECUTE PROCEDURE user_aggregates_user ();

-- post count
CREATE FUNCTION user_aggregates_post_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            user_aggregates
        SET
            post_count = post_count + 1
        WHERE
            user_id = NEW.creator_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            user_aggregates
        SET
            post_count = post_count - 1
        WHERE
            user_id = OLD.creator_id;
        -- If the post gets deleted, the score calculation trigger won't fire,
        -- so you need to re-calculate
        UPDATE
            user_aggregates ua
        SET
            post_score = pd.score
        FROM (
            SELECT
                u.id,
                coalesce(0, sum(pl.score)) AS score
                -- User join because posts could be empty
            FROM
                user_ u
            LEFT JOIN post p ON u.id = p.creator_id
            LEFT JOIN post_like pl ON p.id = pl.post_id
        GROUP BY
            u.id) pd
    WHERE
        ua.user_id = OLD.creator_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER user_aggregates_post_count
    AFTER INSERT OR DELETE ON post
    FOR EACH ROW
    EXECUTE PROCEDURE user_aggregates_post_count ();

-- post score
CREATE FUNCTION user_aggregates_post_score ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Need to get the post creator, not the voter
        UPDATE
            user_aggregates ua
        SET
            post_score = post_score + NEW.score
        FROM
            post p
        WHERE
            ua.user_id = p.creator_id
            AND p.id = NEW.post_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            user_aggregates ua
        SET
            post_score = post_score - OLD.score
        FROM
            post p
        WHERE
            ua.user_id = p.creator_id
            AND p.id = OLD.post_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER user_aggregates_post_score
    AFTER INSERT OR DELETE ON post_like
    FOR EACH ROW
    EXECUTE PROCEDURE user_aggregates_post_score ();

-- comment count
CREATE FUNCTION user_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            user_aggregates
        SET
            comment_count = comment_count + 1
        WHERE
            user_id = NEW.creator_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            user_aggregates
        SET
            comment_count = comment_count - 1
        WHERE
            user_id = OLD.creator_id;
        -- If the comment gets deleted, the score calculation trigger won't fire,
        -- so you need to re-calculate
        UPDATE
            user_aggregates ua
        SET
            comment_score = cd.score
        FROM (
            SELECT
                u.id,
                coalesce(0, sum(cl.score)) AS score
                -- User join because comments could be empty
            FROM
                user_ u
            LEFT JOIN comment c ON u.id = c.creator_id
            LEFT JOIN comment_like cl ON c.id = cl.comment_id
        GROUP BY
            u.id) cd
    WHERE
        ua.user_id = OLD.creator_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER user_aggregates_comment_count
    AFTER INSERT OR DELETE ON comment
    FOR EACH ROW
    EXECUTE PROCEDURE user_aggregates_comment_count ();

-- comment score
CREATE FUNCTION user_aggregates_comment_score ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Need to get the post creator, not the voter
        UPDATE
            user_aggregates ua
        SET
            comment_score = comment_score + NEW.score
        FROM
            comment c
        WHERE
            ua.user_id = c.creator_id
            AND c.id = NEW.comment_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            user_aggregates ua
        SET
            comment_score = comment_score - OLD.score
        FROM
            comment c
        WHERE
            ua.user_id = c.creator_id
            AND c.id = OLD.comment_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER user_aggregates_comment_score
    AFTER INSERT OR DELETE ON comment_like
    FOR EACH ROW
    EXECUTE PROCEDURE user_aggregates_comment_score ();

