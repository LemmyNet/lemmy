CREATE OR REPLACE FUNCTION person_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (was_restored_or_created (TG_OP, OLD, NEW)) THEN
        UPDATE
            person_aggregates
        SET
            comment_count = comment_count + 1
        WHERE
            person_id = NEW.creator_id;
    ELSIF (was_removed_or_deleted (TG_OP, OLD, NEW)) THEN
        UPDATE
            person_aggregates
        SET
            comment_count = comment_count - 1
        WHERE
            person_id = OLD.creator_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE OR REPLACE FUNCTION person_aggregates_post_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (was_restored_or_created (TG_OP, OLD, NEW)) THEN
        UPDATE
            person_aggregates
        SET
            post_count = post_count + 1
        WHERE
            person_id = NEW.creator_id;
    ELSIF (was_removed_or_deleted (TG_OP, OLD, NEW)) THEN
        UPDATE
            person_aggregates
        SET
            post_count = post_count - 1
        WHERE
            person_id = OLD.creator_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE OR REPLACE FUNCTION community_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (was_restored_or_created (TG_OP, OLD, NEW)) THEN
        UPDATE
            community_aggregates ca
        SET
            comments = comments + 1
        FROM
            post p
        WHERE
            p.id = NEW.post_id
            AND ca.community_id = p.community_id;
    ELSIF (was_removed_or_deleted (TG_OP, OLD, NEW)) THEN
        UPDATE
            community_aggregates ca
        SET
            comments = comments - 1
        FROM
            post p
        WHERE
            p.id = OLD.post_id
            AND ca.community_id = p.community_id;
    END IF;
    RETURN NULL;
END
$$;

