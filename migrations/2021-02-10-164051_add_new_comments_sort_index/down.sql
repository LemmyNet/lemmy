DROP INDEX idx_post_aggregates_newest_comment_time, idx_post_aggregates_stickied_newest_comment_time, idx_post_aggregates_stickied_comments;

ALTER TABLE post_aggregates
    DROP COLUMN newest_comment_time;

ALTER TABLE post_aggregates RENAME COLUMN newest_comment_time_necro TO newest_comment_time;

CREATE OR REPLACE FUNCTION post_aggregates_comment_count ()
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

