-- 2023-06-19-120700_no_double_deletion/up.sql
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
        AND ((OLD.deleted = 'f'
                AND NEW.deleted = 't')
            OR (OLD.removed = 'f'
                AND NEW.removed = 't'));
END
$$;

-- 2022-04-04-183652_update_community_aggregates_on_soft_delete/up.sql
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
        AND ((OLD.deleted = 't'
                AND NEW.deleted = 'f')
            OR (OLD.removed = 't'
                AND NEW.removed = 'f'));
END
$$;

-- 2021-08-02-002342_comment_count_fixes/up.sql
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

-- 2020-12-10-152350_create_post_aggregates/up.sql
CREATE OR REPLACE TRIGGER post_aggregates_comment_count
    AFTER INSERT OR DELETE ON comment
    FOR EACH ROW
    EXECUTE PROCEDURE post_aggregates_comment_count ();

