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
CREATE OR REPLACE FUNCTION r.controversy_rank (upvotes numeric, downvotes numeric)
    RETURNS float
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
END;
$$
LANGUAGE plpgsql
IMMUTABLE;

-- Selects both old and new rows in a trigger and allows using `sum(count_diff)` to get the number to add to a count
CREATE FUNCTION r.combine_transition_tables ()
    RETURNS SETOF record
    LANGUAGE sql
    AS $$
    SELECT
        -1 AS count_diff,
        *
    FROM
        old_table
    UNION ALL
    SELECT
        1 AS count_diff,
        *
    FROM
        new_table;
$$;

-- These triggers resolve an item's reports when the item is marked as removed.
CREATE PROCEDURE r.resolve_reports_when_target_removed (target_name text)
LANGUAGE plpgsql
AS $a$
BEGIN
    EXECUTE format($b$ CREATE FUNCTION r.resolve_reports_when_%1$s_removed ( )
            RETURNS TRIGGER
            LANGUAGE plpgsql
            AS $$
            BEGIN
                UPDATE
                    %1$s_report AS report
                SET
                    resolved = TRUE, resolver_id = mod_person_id, updated = now()
                FROM new_removal
                WHERE
                    report.%1$s_id = new_removal.%1$s_id
                    AND new_removal.removed;
                RETURN NULL;
            END $$;
    CREATE TRIGGER resolve_reports
        AFTER INSERT ON mod_remove_%1$s REFERENCING NEW TABLE AS new_removal
        FOR EACH STATEMENT
        EXECUTE FUNCTION r.resolve_reports_when_%1$s_removed ( );
        $b$,
        target_name);
END
$a$;

CALL r.resolve_reports_when_target_removed ('comment');

CALL r.resolve_reports_when_target_removed ('post');

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
        community_id,
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
        id,
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
        new_post
        INNER JOIN community ON community.id = new_post.community_id
    ON CONFLICT
        DO UPDATE SET
            featured_community = excluded.featured_community,
            featured_local = excluded.featured_local;
    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates
    AFTER INSERT OR UPDATE OF featured_community,
    featured_local ON post REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.post_aggregates_from_post ();

CREATE FUNCTION r.site_aggregates_from_site ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- we only ever want to have a single value in site_aggregate because the site_aggregate triggers update all rows in that table.
    -- a cleaner check would be to insert it for the local_site but that would break assumptions at least in the tests
    IF NOT EXISTS (
        SELECT
            1
        FROM
            site_aggregates) THEN
    INSERT INTO site_aggregates (site_id)
        VALUES (NEW.id);
    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON site
    FOR EACH ROW
    EXECUTE FUNCTION r.site_aggregates_from_site ();

-- These triggers update aggregates in response to votes.
CREATE PROCEDURE r.aggregates_from_like (target_name text, creator_id_getter text)
LANGUAGE plpgsql
AS $a$
BEGIN
    EXECUTE format($b$ CREATE FUNCTION r.%1$s_aggregates_from_like ( )
            RETURNS TRIGGER
            LANGUAGE plpgsql
            AS $$
            BEGIN
                -- Update aggregates for target, then update aggregates for target's creator
                WITH target_diff AS ( UPDATE
                        %1$s_aggregates
                    SET
                        (score, upvotes, downvotes, controversy_rank) = (score + diff.upvotes - diff.downvotes, upvotes + diff.upvotes, downvotes + diff.downvotes, controversy_rank ((upvotes + diff.upvotes)::numeric, (downvotes + diff.downvotes)::numeric))
                    FROM (
                        SELECT
                            %1$s_id, sum(count_diff) FILTER (WHERE score = 1) AS upvotes, sum(count_diff) FILTER (WHERE score <> 1) AS downvotes FROM r.combine_transition_tables ()
                GROUP BY %1$s_id) AS diff
                WHERE
                    %1$s_aggregates.%1 $ s_id = diff.%1$s_id
                RETURNING
                    %2$s AS creator_id, diff.upvotes - diff.downvotes AS score)
            UPDATE
                person_aggregates
            SET
                %1$s_score = %1$s_score + diff.sum FROM (
                    SELECT
                        creator_id, sum(score)
                    FROM target_diff GROUP BY creator_id) AS diff
                WHERE
                    person_aggregates.person_id = diff.creator_id;
                RETURN NULL;
            END $$;
    CREATE TRIGGER aggregates
        AFTER INSERT OR DELETE OR UPDATE OF score ON %1$s_like REFERENCING OLD TABLE AS old_table NEW TABLE AS new_table
        FOR EACH STATEMENT
        EXECUTE FUNCTION r.%1$s_aggregates_from_like;
        $b$,
        target_name,
        creator_id_getter);
END
$a$;

CALL r.aggregates_from_like ('comment', '(SELECT creator_id FROM comment WHERE comment.id = target_aggregates.comment_id LIMIT 1)');

CALL r.aggregates_from_like ('post', 'target_aggregates.creator_id');

COMMIT;

