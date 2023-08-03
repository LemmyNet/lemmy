-- First rename current newest comment time to newest_comment_time_necro
-- necro means that time is limited to 2 days, whereas newest_comment_time ignores that.
ALTER TABLE post_aggregates RENAME COLUMN newest_comment_time TO newest_comment_time_necro;

-- Add the newest_comment_time column
ALTER TABLE post_aggregates
    ADD COLUMN newest_comment_time timestamp NOT NULL DEFAULT now();

-- Set the current newest_comment_time based on the old ones
UPDATE
    post_aggregates
SET
    newest_comment_time = newest_comment_time_necro;

-- Add the indexes for this new column
CREATE INDEX idx_post_aggregates_newest_comment_time ON post_aggregates (newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_stickied_newest_comment_time ON post_aggregates (stickied DESC, newest_comment_time DESC);

-- Forgot to add index w/ stickied first for most comments:
CREATE INDEX idx_post_aggregates_stickied_comments ON post_aggregates (stickied DESC, comments DESC);

-- Alter the comment trigger to set the newest_comment_time, and newest_comment_time_necro
CREATE OR REPLACE FUNCTION post_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            post_aggregates pa
        SET
            comments = comments + 1,
            newest_comment_time = NEW.published
        WHERE
            pa.post_id = NEW.post_id;
        -- A 2 day necro-bump limit
        UPDATE
            post_aggregates pa
        SET
            newest_comment_time_necro = NEW.published
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

