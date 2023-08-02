-- Add monthly and half yearly active columns for site and community aggregates
-- These columns don't need to be updated with a trigger, so they're saved daily via queries
ALTER TABLE site_aggregates
    ADD COLUMN users_active_day bigint NOT NULL DEFAULT 0;

ALTER TABLE site_aggregates
    ADD COLUMN users_active_week bigint NOT NULL DEFAULT 0;

ALTER TABLE site_aggregates
    ADD COLUMN users_active_month bigint NOT NULL DEFAULT 0;

ALTER TABLE site_aggregates
    ADD COLUMN users_active_half_year bigint NOT NULL DEFAULT 0;

ALTER TABLE community_aggregates
    ADD COLUMN users_active_day bigint NOT NULL DEFAULT 0;

ALTER TABLE community_aggregates
    ADD COLUMN users_active_week bigint NOT NULL DEFAULT 0;

ALTER TABLE community_aggregates
    ADD COLUMN users_active_month bigint NOT NULL DEFAULT 0;

ALTER TABLE community_aggregates
    ADD COLUMN users_active_half_year bigint NOT NULL DEFAULT 0;

CREATE OR REPLACE FUNCTION site_aggregates_activity (i text)
    RETURNS int
    LANGUAGE plpgsql
    AS $$
DECLARE
    count_ integer;
BEGIN
    SELECT
        count(*) INTO count_
    FROM (
        SELECT
            c.creator_id
        FROM
            comment c
            INNER JOIN user_ u ON c.creator_id = u.id
        WHERE
            c.published > ('now'::timestamp - i::interval)
            AND u.local = TRUE
        UNION
        SELECT
            p.creator_id
        FROM
            post p
            INNER JOIN user_ u ON p.creator_id = u.id
        WHERE
            p.published > ('now'::timestamp - i::interval)
            AND u.local = TRUE) a;
    RETURN count_;
END;
$$;

UPDATE
    site_aggregates
SET
    users_active_day = (
        SELECT
            *
        FROM
            site_aggregates_activity ('1 day'));

UPDATE
    site_aggregates
SET
    users_active_week = (
        SELECT
            *
        FROM
            site_aggregates_activity ('1 week'));

UPDATE
    site_aggregates
SET
    users_active_month = (
        SELECT
            *
        FROM
            site_aggregates_activity ('1 month'));

UPDATE
    site_aggregates
SET
    users_active_half_year = (
        SELECT
            *
        FROM
            site_aggregates_activity ('6 months'));

CREATE OR REPLACE FUNCTION community_aggregates_activity (i text)
    RETURNS TABLE (
        count_ bigint,
        community_id_ integer)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN query
    SELECT
        count(*),
        community_id
    FROM (
        SELECT
            c.creator_id,
            p.community_id
        FROM
            comment c
            INNER JOIN post p ON c.post_id = p.id
        WHERE
            c.published > ('now'::timestamp - i::interval)
        UNION
        SELECT
            p.creator_id,
            p.community_id
        FROM
            post p
        WHERE
            p.published > ('now'::timestamp - i::interval)) a
GROUP BY
    community_id;
END;
$$;

UPDATE
    community_aggregates ca
SET
    users_active_day = mv.count_
FROM
    community_aggregates_activity ('1 day') mv
WHERE
    ca.community_id = mv.community_id_;

UPDATE
    community_aggregates ca
SET
    users_active_week = mv.count_
FROM
    community_aggregates_activity ('1 week') mv
WHERE
    ca.community_id = mv.community_id_;

UPDATE
    community_aggregates ca
SET
    users_active_month = mv.count_
FROM
    community_aggregates_activity ('1 month') mv
WHERE
    ca.community_id = mv.community_id_;

UPDATE
    community_aggregates ca
SET
    users_active_half_year = mv.count_
FROM
    community_aggregates_activity ('6 months') mv
WHERE
    ca.community_id = mv.community_id_;

