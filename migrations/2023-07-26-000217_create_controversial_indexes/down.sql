-- Update comment_aggregates_score trigger function to exclude controversy_rank update
CREATE OR REPLACE FUNCTION comment_aggregates_score ()
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

-- Update post_aggregates_score trigger function to exclude controversy_rank update
CREATE OR REPLACE FUNCTION post_aggregates_score ()
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

-- Drop the indexes
DROP INDEX IF EXISTS idx_post_aggregates_featured_local_controversy;

DROP INDEX IF EXISTS idx_post_aggregates_featured_community_controversy;

DROP INDEX IF EXISTS idx_comment_aggregates_controversy;

-- Remove the added columns from the tables
ALTER TABLE post_aggregates
    DROP COLUMN controversy_rank;

ALTER TABLE comment_aggregates
    DROP COLUMN controversy_rank;

-- Remove function
DROP FUNCTION controversy_rank (numeric, numeric);

