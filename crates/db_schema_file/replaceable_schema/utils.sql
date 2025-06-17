-- Each calculation used in triggers should be a single SQL language
-- expression so it can be inlined in migrations.
CREATE FUNCTION r.controversy_rank (upvotes numeric, downvotes numeric)
    RETURNS float
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN CASE WHEN downvotes <= 0
        OR upvotes <= 0 THEN
        0
    ELSE
        (
            upvotes + downvotes) ^ CASE WHEN upvotes > downvotes THEN
            downvotes::float / upvotes::float
        ELSE
            upvotes::float / downvotes::float
    END
    END;

CREATE FUNCTION r.hot_rank (score numeric, published_at timestamp with time zone)
    RETURNS double precision
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN
    -- after a week, it will default to 0.
    CASE WHEN (
now() - published_at) > '0 days'
        AND (
now() - published_at) < '7 days' THEN
        -- Use greatest(2,score), so that the hot_rank will be positive and not ignored.
        log (
            greatest (2, score + 2)) / power (((EXTRACT(EPOCH FROM (now() - published_at)) / 3600) + 2), 1.8)
    ELSE
        -- if the post is from the future, set hot score to 0. otherwise you can game the post to
        -- always be on top even with only 1 vote by setting it to the future
        0.0
    END;

CREATE FUNCTION r.scaled_rank (score numeric, published_at timestamp with time zone, interactions_month numeric)
    RETURNS double precision
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE
    -- Add 2 to avoid divide by zero errors
    -- Default for score = 1, active users = 1, and now, is (0.1728 / log(2 + 1)) = 0.3621
    -- There may need to be a scale factor multiplied to interactions_month, to make
    -- the log curve less pronounced. This can be tuned in the future.
    RETURN (
        r.hot_rank (score, published_at) / log(2 + interactions_month)
);

-- For tables with `deleted` and `removed` columns, this function determines which rows to include in a count.
CREATE FUNCTION r.is_counted (item record)
    RETURNS bool
    LANGUAGE plpgsql
    IMMUTABLE PARALLEL SAFE
    AS $$
BEGIN
    RETURN COALESCE(NOT (item.deleted
            OR item.removed), FALSE);
END;
$$;

CREATE FUNCTION r.local_url (url_path text)
    RETURNS text
    LANGUAGE sql
    STABLE PARALLEL SAFE RETURN (
current_setting('lemmy.protocol_and_hostname') || url_path
);

-- This function creates statement-level triggers for all operation types. It's designed this way
-- because of these limitations:
--   * A trigger that uses transition tables can only handle 1 operation type.
--   * Transition tables must be relevant for the operation type (for example, `NEW TABLE` is
--     not allowed for a `DELETE` trigger)
--   * Transition tables are only provided to the trigger function, not to functions that it calls.
--
-- This function can only be called once per table. The trigger function body is given as the 2nd argument
-- and can contain these names, which are replaced with a `SELECT` statement in parenthesis if needed:
--   * `select_old_rows`
--   * `select_new_rows`
--   * `select_old_and_new_rows` with 2 columns:
--       1. `count_diff`: `-1` for old rows and `1` for new rows, which can be used with `sum` to get the number
--          to add to a count
--       2. (same name as the trigger's table): the old or new row as a composite value
CREATE PROCEDURE r.create_triggers (table_name text, function_body text)
LANGUAGE plpgsql
AS $a$
DECLARE
    defs text := $$
    -- Delete
    CREATE FUNCTION r.thing_delete_statement ()
        RETURNS TRIGGER
        LANGUAGE plpgsql
        AS function_body_delete;
    CREATE TRIGGER delete_statement
        AFTER DELETE ON thing REFERENCING OLD TABLE AS select_old_rows
        FOR EACH STATEMENT
        EXECUTE FUNCTION r.thing_delete_statement ( );
    -- Insert
    CREATE FUNCTION r.thing_insert_statement ( )
        RETURNS TRIGGER
        LANGUAGE plpgsql
        AS function_body_insert;
    CREATE TRIGGER insert_statement
        AFTER INSERT ON thing REFERENCING NEW TABLE AS select_new_rows
        FOR EACH STATEMENT
        EXECUTE FUNCTION r.thing_insert_statement ( );
    -- Update
    CREATE FUNCTION r.thing_update_statement ( )
        RETURNS TRIGGER
        LANGUAGE plpgsql
        AS function_body_update;
    CREATE TRIGGER update_statement
        AFTER UPDATE ON thing REFERENCING OLD TABLE AS select_old_rows NEW TABLE AS select_new_rows
        FOR EACH STATEMENT
        EXECUTE FUNCTION r.thing_update_statement ( );
    $$;
    select_old_and_new_rows text := $$ (
        SELECT
            -1 AS count_diff,
            old_table::thing AS thing
        FROM
            select_old_rows AS old_table
        UNION ALL
        SELECT
            1 AS count_diff,
            new_table::thing AS thing
        FROM
            select_new_rows AS new_table) $$;
    empty_select_new_rows text := $$ (
        SELECT
            *
        FROM
            -- Real transition table
            select_old_rows
        WHERE
            FALSE) $$;
    empty_select_old_rows text := $$ (
        SELECT
            *
        FROM
            -- Real transition table
            select_new_rows
        WHERE
            FALSE) $$;
    BEGIN
        function_body := replace(function_body, 'select_old_and_new_rows', select_old_and_new_rows);
        -- `select_old_rows` and `select_new_rows` are made available as empty tables if they don't already exist
        defs := replace(defs, 'function_body_delete', quote_literal(replace(function_body, 'select_new_rows', empty_select_new_rows)));
        defs := replace(defs, 'function_body_insert', quote_literal(replace(function_body, 'select_old_rows', empty_select_old_rows)));
        defs := replace(defs, 'function_body_update', quote_literal(function_body));
        defs := replace(defs, 'thing', table_name);
        EXECUTE defs;
END;
$a$;

-- Edit community aggregates to include voters as active users
CREATE OR REPLACE FUNCTION r.community_aggregates_activity (i text)
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
            INNER JOIN person pe ON c.creator_id = pe.id
        WHERE
            c.published_at > ('now'::timestamp - i::interval)
            AND pe.bot_account = FALSE
        UNION
        SELECT
            p.creator_id,
            p.community_id
        FROM
            post p
            INNER JOIN person pe ON p.creator_id = pe.id
        WHERE
            p.published_at > ('now'::timestamp - i::interval)
            AND pe.bot_account = FALSE
        UNION
        SELECT
            pa.person_id,
            p.community_id
        FROM
            post_actions pa
            INNER JOIN post p ON pa.post_id = p.id
            INNER JOIN person pe ON pa.person_id = pe.id
        WHERE
            pa.liked_at > ('now'::timestamp - i::interval)
            AND pe.bot_account = FALSE
        UNION
        SELECT
            ca.person_id,
            p.community_id
        FROM
            comment_actions ca
            INNER JOIN comment c ON ca.comment_id = c.id
            INNER JOIN post p ON c.post_id = p.id
            INNER JOIN person pe ON ca.person_id = pe.id
        WHERE
            ca.liked_at > ('now'::timestamp - i::interval)
            AND pe.bot_account = FALSE) a
GROUP BY
    community_id;
END;
$$;

-- Community aggregate function for adding up total number of interactions
CREATE OR REPLACE FUNCTION r.community_aggregates_interactions (i text)
    RETURNS TABLE (
        count_ bigint,
        community_id_ integer)
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN query
    SELECT
        COALESCE(sum(comments + upvotes + downvotes)::bigint, 0) AS count_,
        community_id AS community_id_
    FROM
        post
    WHERE
        published_at >= (CURRENT_TIMESTAMP - i::interval)
    GROUP BY
        community_id;
END;
$$;

-- Edit site aggregates to include voters and people who have read posts as active users
CREATE OR REPLACE FUNCTION r.site_aggregates_activity (i text)
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
            INNER JOIN person pe ON c.creator_id = pe.id
        WHERE
            c.published_at > ('now'::timestamp - i::interval)
            AND pe.local = TRUE
            AND pe.bot_account = FALSE
        UNION
        SELECT
            p.creator_id
        FROM
            post p
            INNER JOIN person pe ON p.creator_id = pe.id
        WHERE
            p.published_at > ('now'::timestamp - i::interval)
            AND pe.local = TRUE
            AND pe.bot_account = FALSE
        UNION
        SELECT
            pa.person_id
        FROM
            post_actions pa
            INNER JOIN person pe ON pa.person_id = pe.id
        WHERE
            pa.liked_at > ('now'::timestamp - i::interval)
            AND pe.local = TRUE
            AND pe.bot_account = FALSE
        UNION
        SELECT
            ca.person_id
        FROM
            comment_actions ca
            INNER JOIN person pe ON ca.person_id = pe.id
        WHERE
            ca.liked_at > ('now'::timestamp - i::interval)
            AND pe.local = TRUE
            AND pe.bot_account = FALSE) a;
    RETURN count_;
END;
$$;

