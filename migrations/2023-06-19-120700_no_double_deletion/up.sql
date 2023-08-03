-- Deleting after removing should not decrement the count twice.
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

-- Recalculate proper comment count.
UPDATE
    person_aggregates
SET
    comment_count = cnt.count
FROM (
    SELECT
        creator_id,
        count(*) AS count
    FROM
        comment
    WHERE
        deleted = 'f'
        AND removed = 'f'
    GROUP BY
        creator_id) cnt
WHERE
    person_aggregates.person_id = cnt.creator_id;

-- Recalculate proper comment score.
UPDATE
    person_aggregates ua
SET
    comment_score = cd.score
FROM (
    SELECT
        u.id AS creator_id,
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
    ua.person_id = cd.creator_id;

-- Recalculate proper post count.
UPDATE
    person_aggregates
SET
    post_count = cnt.count
FROM (
    SELECT
        creator_id,
        count(*) AS count
    FROM
        post
    WHERE
        deleted = 'f'
        AND removed = 'f'
    GROUP BY
        creator_id) cnt
WHERE
    person_aggregates.person_id = cnt.creator_id;

-- Recalculate proper post score.
UPDATE
    person_aggregates ua
SET
    post_score = pd.score
FROM (
    SELECT
        u.id AS creator_id,
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
    ua.person_id = pd.creator_id;

