-- Fix for duplicated decrementations when both `deleted` and `removed` fields are set subsequently
CREATE OR REPLACE FUNCTION was_removed_or_deleted (TG_OP text, OLD record, NEW record)
    RETURNS boolean
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        RETURN FALSE;
    END IF;
    IF (TG_OP = 'DELETE' AND OLD.deleted = 'f' AND OLD.removed = 'f') THEN
        RETURN TRUE;
    END IF;
    RETURN TG_OP = 'UPDATE'
        AND OLD.deleted = 'f'
        AND OLD.removed = 'f'
        AND (NEW.deleted = 't'
            OR NEW.removed = 't');
END
$$;

CREATE OR REPLACE FUNCTION was_restored_or_created (TG_OP text, OLD record, NEW record)
    RETURNS boolean
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        RETURN FALSE;
    END IF;
    IF (TG_OP = 'INSERT') THEN
        RETURN TRUE;
    END IF;
    RETURN TG_OP = 'UPDATE'
        AND NEW.deleted = 'f'
        AND NEW.removed = 'f'
        AND (OLD.deleted = 't'
            OR OLD.removed = 't');
END
$$;

-- Fix for post's comment count not updating after setting `removed` to 't'
DROP TRIGGER IF EXISTS post_aggregates_comment_set_deleted ON comment;

DROP FUNCTION post_aggregates_comment_deleted ();

CREATE OR REPLACE FUNCTION post_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Check for post existence - it may not exist anymore
    IF TG_OP = 'INSERT' OR EXISTS (
        SELECT
            1
        FROM
            post p
        WHERE
            p.id = OLD.post_id) THEN
        IF (was_restored_or_created (TG_OP, OLD, NEW)) THEN
            UPDATE
                post_aggregates pa
            SET
                comments = comments + 1
            WHERE
                pa.post_id = NEW.post_id;
        ELSIF (was_removed_or_deleted (TG_OP, OLD, NEW)) THEN
            UPDATE
                post_aggregates pa
            SET
                comments = comments - 1
            WHERE
                pa.post_id = OLD.post_id;
        END IF;
    END IF;
    IF TG_OP = 'INSERT' THEN
        UPDATE
            post_aggregates pa
        SET
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
    END IF;
    RETURN NULL;
END
$$;

CREATE OR REPLACE TRIGGER post_aggregates_comment_count
    AFTER INSERT OR DELETE OR UPDATE OF removed,
    deleted ON comment
    FOR EACH ROW
    EXECUTE PROCEDURE post_aggregates_comment_count ();

