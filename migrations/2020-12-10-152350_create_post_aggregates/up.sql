-- Add post aggregates
CREATE TABLE post_aggregates (
    id serial PRIMARY KEY,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    comments bigint NOT NULL DEFAULT 0,
    score bigint NOT NULL DEFAULT 0,
    upvotes bigint NOT NULL DEFAULT 0,
    downvotes bigint NOT NULL DEFAULT 0,
    stickied boolean NOT NULL DEFAULT FALSE,
    published timestamp NOT NULL DEFAULT now(),
    newest_comment_time timestamp NOT NULL DEFAULT now(),
    UNIQUE (post_id)
);

INSERT INTO post_aggregates (post_id, comments, score, upvotes, downvotes, stickied, published, newest_comment_time)
SELECT
    p.id,
    coalesce(ct.comments, 0::bigint) AS comments,
    coalesce(pl.score, 0::bigint) AS score,
    coalesce(pl.upvotes, 0::bigint) AS upvotes,
    coalesce(pl.downvotes, 0::bigint) AS downvotes,
    p.stickied,
    p.published,
    greatest (ct.recent_comment_time, p.published) AS newest_activity_time
FROM
    post p
    LEFT JOIN (
        SELECT
            comment.post_id,
            count(*) AS comments,
            max(comment.published) AS recent_comment_time
        FROM
            comment
        GROUP BY
            comment.post_id) ct ON ct.post_id = p.id
    LEFT JOIN (
        SELECT
            post_like.post_id,
            sum(post_like.score) AS score,
            sum(post_like.score) FILTER (WHERE post_like.score = 1) AS upvotes,
            - sum(post_like.score) FILTER (WHERE post_like.score = '-1'::integer) AS downvotes
        FROM
            post_like
        GROUP BY
            post_like.post_id) pl ON pl.post_id = p.id;

-- Add community aggregate triggers
-- initial post add
CREATE FUNCTION post_aggregates_post ()
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

CREATE TRIGGER post_aggregates_post
    AFTER INSERT OR DELETE ON post
    FOR EACH ROW
    EXECUTE PROCEDURE post_aggregates_post ();

-- comment count
CREATE FUNCTION post_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            post_aggregates pa
        SET
            comments = comments + 1
        WHERE
            pa.post_id = NEW.post_id;
        -- A 2 day necro-bump limit
        UPDATE
            post_aggregates pa
        SET
            newest_comment_time = NEW.published
        WHERE
            pa.post_id = NEW.post_id
            AND published > ('now'::timestamp - '2 days'::interval);
    ELSIF (TG_OP = 'DELETE') THEN
        -- Join to post because that post may not exist anymore
        UPDATE
            post_aggregates pa
        SET
            comments = comments - 1
        FROM
            post p
        WHERE
            pa.post_id = p.id
            AND pa.post_id = OLD.post_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER post_aggregates_comment_count
    AFTER INSERT OR DELETE ON comment
    FOR EACH ROW
    EXECUTE PROCEDURE post_aggregates_comment_count ();

-- post score
CREATE FUNCTION post_aggregates_score ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            post_aggregates pa
        SET
            score = score + NEW.score,
            upvotes = CASE WHEN NEW.score = 1 THEN
                upvotes + 1
            ELSE
                upvotes
            END,
            downvotes = CASE WHEN NEW.score = - 1 THEN
                downvotes + 1
            ELSE
                downvotes
            END
        WHERE
            pa.post_id = NEW.post_id;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Join to post because that post may not exist anymore
        UPDATE
            post_aggregates pa
        SET
            score = score - OLD.score,
            upvotes = CASE WHEN OLD.score = 1 THEN
                upvotes - 1
            ELSE
                upvotes
            END,
            downvotes = CASE WHEN OLD.score = - 1 THEN
                downvotes - 1
            ELSE
                downvotes
            END
        FROM
            post p
        WHERE
            pa.post_id = p.id
            AND pa.post_id = OLD.post_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER post_aggregates_score
    AFTER INSERT OR DELETE ON post_like
    FOR EACH ROW
    EXECUTE PROCEDURE post_aggregates_score ();

-- post stickied
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

