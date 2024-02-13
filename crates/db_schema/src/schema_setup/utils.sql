-- Each calculation used in triggers should be a single SQL language
-- expression so it can be inlined in migrations.
CREATE FUNCTION r.controversy_rank (upvotes numeric, downvotes numeric)
    RETURNS float
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE
    RETURN
    CASE WHEN downvotes <= 0 OR upvotes <= 0 THEN
        0
    ELSE
        (upvotes + downvotes) * CASE WHEN upvotes > downvotes THEN
            downvotes::float / upvotes::float
        ELSE
            upvotes::float / downvotes::float
        END
    END;

-- For tables with `deleted` and `removed` columns, this function determines which rows to include in a count.
CREATE FUNCTION r.is_counted (item record)
    RETURNS bool
    LANGUAGE plpgsql
    IMMUTABLE PARALLEL SAFE
    AS $$
BEGIN
    RETURN NOT (item.deleted
        OR item.removed);
END;
$$;

-- This function creates statement-level triggers for all operation types. It's designed this way
-- because of these limitations:
--   * A trigger that uses transition tables can only handle 1 operation type.
--   * Transition tables must be relevant for the operation type (for example, `NEW TABLE` is
--     not allowed for a `DELETE` trigger)
--   * Transition tables are only provided to the trigger function, not to functions that it calls.
--
-- This function can only be called once per table. The trigger function body given as the 2nd argument
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

