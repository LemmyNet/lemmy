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
        -- If the comment gets deleted, the score calculation trigger won't fire,
        -- so you need to re-calculate
        UPDATE
            person_aggregates ua
        SET
            comment_score = cd.score
        FROM (
            SELECT
                u.id,
                coalesce(0, sum(cl.score)) AS score
                -- User join because comments could be empty
            FROM
                person u
            LEFT JOIN comment c ON u.id = c.creator_id
                AND c.deleted = 'f'
                AND c.removed = 'f'
        LEFT JOIN comment_like cl ON c.id = cl.comment_id
    GROUP BY
        u.id) cd
    WHERE
        ua.person_id = OLD.creator_id;
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
        -- If the post gets deleted, the score calculation trigger won't fire,
        -- so you need to re-calculate
        UPDATE
            person_aggregates ua
        SET
            post_score = pd.score
        FROM (
            SELECT
                u.id,
                coalesce(0, sum(pl.score)) AS score
                -- User join because posts could be empty
            FROM
                person u
            LEFT JOIN post p ON u.id = p.creator_id
                AND p.deleted = 'f'
                AND p.removed = 'f'
        LEFT JOIN post_like pl ON p.id = pl.post_id
    GROUP BY
        u.id) pd
    WHERE
        ua.person_id = OLD.creator_id;
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
            comment c,
            post p
        WHERE
            p.id = c.post_id
            AND p.id = NEW.post_id
            AND ca.community_id = p.community_id;
    ELSIF (was_removed_or_deleted (TG_OP, OLD, NEW)) THEN
        UPDATE
            community_aggregates ca
        SET
            comments = comments - 1
        FROM
            comment c,
            post p
        WHERE
            p.id = c.post_id
            AND p.id = OLD.post_id
            AND ca.community_id = p.community_id;
    END IF;
    RETURN NULL;
END
$$;

