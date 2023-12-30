-- This sets up the `r` schema, which contains things that can be safely dropped and replaced instead of being
-- changed using migrations.
--
-- Statements in this file may not create or modify things outside of the `r` schema (indicated by the `r.` prefix),
-- except for these things, which are associated with something other than a schema (usually a table):
--   * A trigger if the function name after `EXECUTE FUNCTION` is in `r` (dropping `r` drops the trigger)
--
-- The default schema is not temporarily set to `r` because it would not affect some things (such as triggers) which
-- makes it hard to tell if the rule above is being followed.
--
-- If you add something here that depends on something (such as a table) created in a new migration, then down.sql must use
-- `CASCADE` when dropping it. This doesn't need to be fixed in old migrations because the "replaceable-schema" migration
-- runs `DROP SCHEMA IF EXISTS r CASCADE` in down.sql.
BEGIN;

DROP SCHEMA IF EXISTS r CASCADE;

CREATE SCHEMA r;

-- Rank calculations
CREATE FUNCTION r.controversy_rank (upvotes numeric, downvotes numeric)
    RETURNS float
    LANGUAGE plpgsql
    IMMUTABLE PARALLEL SAFE
    AS $$
BEGIN
    IF downvotes <= 0 OR upvotes <= 0 THEN
        RETURN 0;
    ELSE
        RETURN (upvotes + downvotes) * CASE WHEN upvotes > downvotes THEN
            downvotes::float / upvotes::float
        ELSE
            upvotes::float / downvotes::float
        END;
    END IF;
END
$$;

-- This function creates statement-level triggers for all operation types. It's designed this way
-- because of these limitations:
--   * A trigger that uses transition tables can only handle 1 operation type.
--   * Transition tables must be relevant for the operation type (for example, `NEW TABLE` is
--     not allowed for a `DELETE` trigger)
--   * Transition tables are only provided to the trigger function, not to functions that it calls.
--
-- This function can only be called once per table. The command to run is given as the 2nd argument
-- and can use these tables in `FROM` clauses:
--   * `old_table` with old rows (alias that doesn't contain `old_table` must be added)
--   * `new_table` with new rows (alias that doesn't contain `new_table` must be added)
--   * `combined_transition_tables` with both old and new rows, with 2 columns:
--       1. `count_diff`: `-1` for old rows and `1` for new rows, which can be used with `sum` to get the number
--          to add to a count
--       2. (same name as the trigger's table): the old or new row as a composite value
CREATE PROCEDURE r.create_triggers (table_name text, command text)
LANGUAGE plpgsql
AS $a$
DECLARE
    defs text := $b$
    -- Delete
    CREATE FUNCTION r.thing_delete_statement ()
        RETURNS TRIGGER
        LANGUAGE plpgsql
        AS $$
        BEGIN
            delete_command;
            RETURN NULL;
        END $$;
        CREATE TRIGGER delete_statement
            AFTER DELETE ON thing REFERENCING OLD TABLE AS old_table
            FOR EACH STATEMENT
            EXECUTE FUNCTION r.thing_delete_statement ( );
        -- Insert
        CREATE FUNCTION r.thing_insert_statement ( )
            RETURNS TRIGGER
            LANGUAGE plpgsql
            AS $$
            BEGIN
                insert_command;
                RETURN NULL;
            END $$;
        CREATE TRIGGER insert_statement
            AFTER INSERT ON thing REFERENCING NEW TABLE AS new_table
            FOR EACH STATEMENT
            EXECUTE FUNCTION r.thing_insert_statement ( );
        -- Update
        CREATE FUNCTION r.thing_update_statement ( )
            RETURNS TRIGGER
            LANGUAGE plpgsql
            AS $$
            BEGIN
                update_command;
                RETURN NULL;
            END $$;
        CREATE TRIGGER update_statement
            AFTER UPDATE ON thing REFERENCING OLD TABLE AS old_table NEW TABLE AS new_table
            FOR EACH STATEMENT
            EXECUTE FUNCTION r.thing_update_statement ( );
        $b$;
        BEGIN
            -- Couldn't get these to work using CTE
            command := replace(command, 'combined_transition_tables', $$ (
                    SELECT
                        -1 AS count_diff, old_t::thing AS thing
                    FROM old_table AS old_t
                UNION ALL
                SELECT
                    1 AS count_diff, new_t::thing AS thing
                FROM new_table AS new_t) AS combined_transition_tables $$);
            -- `new_table` and `old_table` are made available as empty tables if they don't already exist
            defs := replace(defs, 'delete_command', replace(command, 'new_table', $$ (
                        SELECT
                            *
                        FROM old_table
                        WHERE
                            FALSE) $$));
            defs := replace(defs, 'insert_command', replace(command, 'old_table', $$ (
                        SELECT
                            *
                        FROM new_table
                        WHERE
                            FALSE) $$));
            defs := replace(defs, 'update_command', command);
            defs := replace(defs, 'thing', table_name);
            EXECUTE defs;
        END
$a$;

-- Define functions
CREATE FUNCTION r.creator_id_from_post_aggregates (agg post_aggregates)
    RETURNS int RETURN agg.creator_id;

CREATE FUNCTION r.creator_id_from_comment_aggregates (agg comment_aggregates)
    RETURNS int RETURN (
        SELECT
            creator_id
        FROM
            comment
        WHERE
            comment.id = agg.comment_id LIMIT 1
);

-- Create triggers for both post and comments
CREATE PROCEDURE r.post_or_comment (table_name text)
LANGUAGE plpgsql
AS $a$
BEGIN
    EXECUTE replace($b$
        -- When a thing is removed, resolve its reports
        CREATE FUNCTION r.resolve_reports_when_thing_removed ( )
            RETURNS TRIGGER
            LANGUAGE plpgsql
            AS $$
            BEGIN
                UPDATE
                    thing_report
                SET
                    resolved = TRUE, resolver_id = first_removal.mod_person_id, updated = first_removal.when_ FROM ( SELECT DISTINCT
                            thing_id
                        FROM new_removal) AS removal_group, LATERAL (
                    SELECT
                        *
                    FROM new_removal
                    WHERE
                        new_removal.thing_id = removal_group.thing_id ORDER BY when_ ASC LIMIT 1) AS first_removal
            WHERE
                thing_report.thing_id = first_removal.thing_id
                    AND NOT thing_report.resolved
                    AND COALESCE(thing_report.updated < first_removal.when_, TRUE);
                RETURN NULL;
            END $$;
    CREATE TRIGGER resolve_reports
        AFTER INSERT ON mod_remove_thing REFERENCING NEW TABLE AS new_removal
        FOR EACH STATEMENT
        EXECUTE FUNCTION r.resolve_reports_when_thing_removed ( );
        -- When a thing gets a vote, update its aggregates and its creator's aggregates
        CALL r.create_triggers ('thing_like', $$ WITH thing_diff AS ( UPDATE
                    thing_aggregates AS a
                SET
                    score = a.score + diff.upvotes - diff.downvotes, upvotes = a.upvotes + diff.upvotes, downvotes = a.downvotes + diff.downvotes, controversy_rank = controversy_rank ((a.upvotes + diff.upvotes)::numeric, (a.downvotes + diff.downvotes)::numeric)
                FROM (
                    SELECT
                        (thing_like).thing_id, coalesce(sum(count_diff) FILTER (WHERE (thing_like).score = 1), 0) AS upvotes, coalesce(sum(count_diff) FILTER (WHERE (thing_like).score != 1), 0) AS downvotes FROM combined_transition_tables GROUP BY (thing_like).thing_id) AS diff
        WHERE
            a.thing_id = diff.thing_id
        RETURNING
            r.creator_id_from_thing_aggregates (a.*) AS creator_id, diff.upvotes - diff.downvotes AS score)
    UPDATE
        person_aggregates AS a
    SET
        thing_score = a.thing_score + diff.score FROM (
            SELECT
                creator_id, sum(score) AS score FROM thing_diff GROUP BY creator_id) AS diff
        WHERE
            a.person_id = diff.creator_id $$);
        $b$,
        'thing',
        table_name);
END
$a$;

CALL r.post_or_comment ('post');

CALL r.post_or_comment ('comment');

-- Create triggers that update counts in parent aggregates
CALL r.create_triggers ('comment', $$ WITH comment_group AS (
        SELECT
            (comment).post_id,
            (comment).creator_id,
            (comment).local,
            coalesce(sum(count_diff), 0) AS comments FROM combined_transition_tables
            WHERE
                NOT ((comment).deleted
                OR (comment).removed)
        GROUP BY GROUPING SETS ((comment).post_id, (comment).creator_id, (comment).local)),
unused_person_aggregates_update_result AS (
    UPDATE
        person_aggregates AS a
    SET
        comment_count = a.comment_count + comment_group.comments
    FROM
        comment_group
    WHERE
        a.person_id = comment_group.creator_id),
    unused_site_aggregates_update_result AS (
        UPDATE
            site_aggregates AS a
        SET
            comments = a.comments + comment_group.comments
        FROM
            comment_group
        WHERE
            comment_group.local),
        post_diff AS (
            UPDATE
                post_aggregates AS a
            SET
                comments = a.comments + comment_group.comments,
                newest_comment_time = GREATEST (a.newest_comment_time, (
                        SELECT
                            max(published)
                        FROM new_table AS new_comment
                        WHERE
                            a.post_id = new_comment.post_id LIMIT 1)),
                newest_comment_time_necro = GREATEST (a.newest_comment_time_necro, (
                        SELECT
                            max(published)
                        FROM new_table AS new_comment
                        WHERE
                            a.post_id = new_comment.post_id
                            -- Ignore comments from the post's creator
                            AND a.creator_id != new_comment.creator_id
                            -- Ignore comments on old posts
                            AND a.published > (new_comment.published - '2 days'::interval)
                        LIMIT 1))
            FROM
                comment_group
            WHERE
                a.post_id = comment_group.post_id
            RETURNING
                a.community_id,
                comment_group.comments,
                (
                    SELECT
                        NOT (post.deleted
                            OR post.removed)
                    FROM
                        post
                    WHERE
                        a.post_id = post.id
                    LIMIT 1) AS include_in_community_aggregates)
        UPDATE
            community_aggregates AS a
        SET
            comments = a.comments + diff.comments
        FROM (
            SELECT
                community_id, sum(comments) AS comments
            FROM
                post_diff
            WHERE
                post_diff.include_in_community_aggregates
            GROUP BY
                community_id) AS diff
        WHERE
            a.community_id = diff.community_id $$);

CALL r.create_triggers ('post', $$ WITH post_group AS (
        SELECT
            (post).community_id, (post).creator_id, (post).local, coalesce(sum(count_diff), 0) AS posts FROM combined_transition_tables
            WHERE
                NOT ((post).deleted
                OR (post).removed)
        GROUP BY GROUPING SETS ((post).community_id, (post).creator_id, (post).local)
), unused_person_aggregates_update_result AS ( UPDATE
        person_aggregates AS a
    SET
        post_count = a.post_count + post_group.posts FROM post_group
        WHERE
            a.person_id = post_group.creator_id
), unused_site_aggregates_update_result AS ( UPDATE
        site_aggregates AS a
    SET
        posts = a.posts + post_group.posts FROM post_group
        WHERE
            post_group.local)
    UPDATE
        community_aggregates AS a
    SET
        posts = a.posts + post_group.posts FROM post_group
        WHERE
            a.community_id = post_group.community_id $$);

CALL r.create_triggers ('community', $$ UPDATE
        site_aggregates AS a
    SET
        communities = a.communities + diff.communities FROM (
            SELECT
                coalesce(sum(count_diff), 0) AS communities FROM combined_transition_tables
            WHERE (community).local
            AND NOT ((community).deleted
            OR (community).removed)) AS diff $$);

CALL r.create_triggers ('person', $$ UPDATE
        site_aggregates AS a
    SET
        users = a.users + diff.users FROM (
            SELECT
                coalesce(sum(count_diff), 0) AS users FROM combined_transition_tables
            WHERE (person).local) AS diff $$);

-- For community_aggregates.comments, don't include comments of deleted or removed posts
CREATE FUNCTION r.update_comment_count_from_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        community_aggregates AS a
    SET
        comments = a.comments + diff.comments
    FROM (
        SELECT
            old_post.community_id,
            sum((
                CASE WHEN new_post.deleted
                    AND new_post.removed THEN
                    -1
                ELSE
                    1
                END) * post_aggregates.comments) AS comments
        FROM
            new_post
            INNER JOIN old_post ON new_post.id = old_post.id
                AND (new_post.deleted
                    AND new_post.removed) != (old_post.deleted
                    AND old_post.removed),
                LATERAL (
                    SELECT
                        *
                    FROM
                        post_aggregates
                WHERE
                    post_id = new_post.id
                LIMIT 1) AS post_aggregates
        GROUP BY
            old_post.community_id) AS diff
WHERE
    a.community_id = diff.community_id;
    RETURN NULL;
END
$$;

CREATE TRIGGER comment_count
    AFTER UPDATE ON post REFERENCING OLD TABLE AS old_post NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.update_comment_count_from_post ();

-- Count subscribers for local communities
CALL r.create_triggers ('community_follower', $$ UPDATE
        community_aggregates AS a
    SET
        subscribers = a.subscribers + diff.subscribers FROM (
            SELECT
                (community_follower).community_id, coalesce(sum(count_diff), 0) AS subscribers FROM combined_transition_tables
            WHERE (
                SELECT
                    local
                FROM community
                WHERE
                    community.id = (community_follower).community_id LIMIT 1)
    GROUP BY (community_follower).community_id) AS diff
    WHERE
        a.community_id = diff.community_id $$);

-- These triggers create and update rows in each aggregates table to match its associated table's rows.
-- Deleting rows and updating IDs are already handled by `CASCADE` in foreign key constraints.
CREATE FUNCTION r.comment_aggregates_from_comment ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO comment_aggregates (comment_id, published)
    SELECT
        id,
        published
    FROM
        new_comment;
    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON comment REFERENCING NEW TABLE AS new_comment
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.comment_aggregates_from_comment ();

CREATE FUNCTION r.community_aggregates_from_community ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO community_aggregates (community_id, published)
    SELECT
        id,
        published
    FROM
        new_community;
    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON community REFERENCING NEW TABLE AS new_community
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.community_aggregates_from_community ();

CREATE FUNCTION r.person_aggregates_from_person ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO person_aggregates (person_id)
    SELECT
        id
    FROM
        new_person;
    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON person REFERENCING NEW TABLE AS new_person
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.person_aggregates_from_person ();

CREATE FUNCTION r.post_aggregates_from_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO post_aggregates (post_id, published, newest_comment_time, newest_comment_time_necro, community_id, creator_id, instance_id, featured_community, featured_local)
    SELECT
        new_post.id,
        new_post.published,
        new_post.published,
        new_post.published,
        new_post.community_id,
        new_post.creator_id,
        community.instance_id,
        new_post.featured_community,
        new_post.featured_local
    FROM
        new_post,
        LATERAL (
            SELECT
                *
            FROM
                community
            WHERE
                community.id = new_post.community_id
            LIMIT 1) AS community;
    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON post REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.post_aggregates_from_post ();

CREATE FUNCTION r.post_aggregates_from_post_update ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        post_aggregates
    SET
        featured_community = new_post.featured_community,
        featured_local = new_post.featured_local
    FROM
        new_post
    WHERE
        post_aggregates.post_id = new_post.id;
    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates_update
    AFTER UPDATE ON post REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.post_aggregates_from_post_update ();

CREATE FUNCTION r.site_aggregates_from_site ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- we only ever want to have a single value in site_aggregate because the site_aggregate triggers update all rows in that table.
    -- a cleaner check would be to insert it for the local_site but that would break assumptions at least in the tests
    IF (NOT EXISTS (
        SELECT
            1
        FROM
            site_aggregates)) THEN
        INSERT INTO site_aggregates (site_id)
            VALUES (NEW.id);
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON site
    FOR EACH ROW
    EXECUTE FUNCTION r.site_aggregates_from_site ();

COMMIT;

