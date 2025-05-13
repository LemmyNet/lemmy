-- Add comment aggregates
CREATE TABLE comment_aggregates (
    id serial PRIMARY KEY,
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    score bigint NOT NULL DEFAULT 0,
    upvotes bigint NOT NULL DEFAULT 0,
    downvotes bigint NOT NULL DEFAULT 0,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (comment_id)
);

INSERT INTO comment_aggregates (comment_id, score, upvotes, downvotes, published)
SELECT
    c.id,
    COALESCE(cl.total, 0::bigint) AS score,
    COALESCE(cl.up, 0::bigint) AS upvotes,
    COALESCE(cl.down, 0::bigint) AS downvotes,
    c.published
FROM
    comment c
    LEFT JOIN (
        SELECT
            l.comment_id AS id,
            sum(l.score) AS total,
            count(
                CASE WHEN l.score = 1 THEN
                    1
                ELSE
                    NULL::integer
                END) AS up,
            count(
                CASE WHEN l.score = '-1'::integer THEN
                    1
                ELSE
                    NULL::integer
                END) AS down
        FROM
            comment_like l
        GROUP BY
            l.comment_id) cl ON cl.id = c.id;

-- Add comment aggregate triggers
-- initial comment add
CREATE FUNCTION comment_aggregates_comment ()
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

CREATE TRIGGER comment_aggregates_comment
    AFTER INSERT OR DELETE ON comment
    FOR EACH ROW
    EXECUTE PROCEDURE comment_aggregates_comment ();

-- comment score
CREATE FUNCTION comment_aggregates_score ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            comment_aggregates ca
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
            ca.comment_id = NEW.comment_id;
    ELSIF (TG_OP = 'DELETE') THEN
        -- Join to comment because that comment may not exist anymore
        UPDATE
            comment_aggregates ca
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
            comment c
        WHERE
            ca.comment_id = c.id
            AND ca.comment_id = OLD.comment_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER comment_aggregates_score
    AFTER INSERT OR DELETE ON comment_like
    FOR EACH ROW
    EXECUTE PROCEDURE comment_aggregates_score ();

