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

CREATE OR REPLACE FUNCTION site_aggregates_activity (i text)
    RETURNS integer
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
            INNER JOIN person u ON c.creator_id = u.id
        WHERE
            c.published > ('now'::timestamp - i::interval)
            AND u.local = TRUE
        UNION
        SELECT
            p.creator_id
        FROM
            post p
            INNER JOIN person u ON p.creator_id = u.id
        WHERE
            p.published > ('now'::timestamp - i::interval)
            AND u.local = TRUE) a;
    RETURN count_;
END;
$$;

