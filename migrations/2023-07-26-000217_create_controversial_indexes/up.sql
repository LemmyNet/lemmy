-- Need to add immutable to the controversy_rank function in order to index by it
-- Controversy Rank:
--      if downvotes <= 0 or upvotes <= 0:
--          0
--      else:
--          (upvotes + downvotes) * min(upvotes, downvotes) / max(upvotes, downvotes)
CREATE OR REPLACE FUNCTION controversy_rank (upvotes numeric, downvotes numeric)
    RETURNS float
    AS $$
BEGIN
    IF downvotes <= 0 OR upvotes <= 0 THEN
        RETURN 0;
    ELSE
        RETURN (upvotes + downvotes) * CASE WHEN upvotes > downvotes THEN
            downvotes::float / upvotes::float
        ELSE
            upvotes::float / downvotes::float
        END;
    END IF;
END;
$$
LANGUAGE plpgsql
IMMUTABLE;

-- Aggregates
ALTER TABLE post_aggregates
    ADD COLUMN controversy_rank float NOT NULL DEFAULT 0;

ALTER TABLE comment_aggregates
    ADD COLUMN controversy_rank float NOT NULL DEFAULT 0;

-- Populate them initially
-- Note: After initial population, these are updated with vote triggers
UPDATE
    post_aggregates
SET
    controversy_rank = controversy_rank (upvotes::numeric, downvotes::numeric);

UPDATE
    comment_aggregates
SET
    controversy_rank = controversy_rank (upvotes::numeric, downvotes::numeric);

-- Create single column indexes
CREATE INDEX idx_post_aggregates_featured_local_controversy ON post_aggregates (featured_local DESC, controversy_rank DESC);

CREATE INDEX idx_post_aggregates_featured_community_controversy ON post_aggregates (featured_community DESC, controversy_rank DESC);

CREATE INDEX idx_comment_aggregates_controversy ON comment_aggregates (controversy_rank DESC);

-- Update post_aggregates_score trigger function to include controversy_rank update
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
            END,
            controversy_rank = controversy_rank (pa.upvotes + CASE WHEN NEW.score = 1 THEN
                    1
                ELSE
                    0
                END::numeric, pa.downvotes + CASE WHEN NEW.score = - 1 THEN
                    1
                ELSE
                    0
                END::numeric)
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
            END,
            controversy_rank = controversy_rank (pa.upvotes + CASE WHEN NEW.score = 1 THEN
                    1
                ELSE
                    0
                END::numeric, pa.downvotes + CASE WHEN NEW.score = - 1 THEN
                    1
                ELSE
                    0
                END::numeric)
        FROM
            post p
        WHERE
            pa.post_id = p.id
            AND pa.post_id = OLD.post_id;
    END IF;
    RETURN NULL;
END
$$;

-- Update comment_aggregates_score trigger function to include controversy_rank update
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
            END,
            controversy_rank = controversy_rank (ca.upvotes + CASE WHEN NEW.score = 1 THEN
                    1
                ELSE
                    0
                END::numeric, ca.downvotes + CASE WHEN NEW.score = - 1 THEN
                    1
                ELSE
                    0
                END::numeric)
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
            END,
            controversy_rank = controversy_rank (ca.upvotes + CASE WHEN NEW.score = 1 THEN
                    1
                ELSE
                    0
                END::numeric, ca.downvotes + CASE WHEN NEW.score = - 1 THEN
                    1
                ELSE
                    0
                END::numeric)
        FROM
            comment c
        WHERE
            ca.comment_id = c.id
            AND ca.comment_id = OLD.comment_id;
    END IF;
    RETURN NULL;
END
$$;

