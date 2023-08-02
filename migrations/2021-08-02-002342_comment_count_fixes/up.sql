-- Creating a new trigger for when comment.deleted is updated
CREATE OR REPLACE FUNCTION post_aggregates_comment_deleted ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.deleted = TRUE THEN
        UPDATE
            post_aggregates pa
        SET
            comments = comments - 1
        WHERE
            pa.post_id = NEW.post_id;
    ELSE
        UPDATE
            post_aggregates pa
        SET
            comments = comments + 1
        WHERE
            pa.post_id = NEW.post_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER post_aggregates_comment_set_deleted
    AFTER UPDATE OF deleted ON comment
    FOR EACH ROW
    EXECUTE PROCEDURE post_aggregates_comment_deleted ();

-- Fix issue with being able to necro-bump your own post
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
        FROM
            post p
        WHERE
            pa.post_id = p.id
            AND pa.post_id = NEW.post_id
            -- Fix issue with being able to necro-bump your own post
            AND NEW.creator_id != p.creator_id
            AND pa.published > ('now'::timestamp - '2 days'::interval);
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
    ELSIF (TG_OP = 'UPDATE') THEN
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

