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

-- These triggers resolve an item's reports when the item is marked as removed.

CREATE FUNCTION resolve_reports_when_post_removed () RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        post_report
    SET
        resolved = TRUE,
        resolver_id = mod_person_id,
        updated = now()
    FROM
        new_removal
    WHERE
        post_report.post_id = new_removal.post_id AND new_removal.removed

    RETURN NULL;
END
$$;

CREATE TRIGGER resolve_reports
    AFTER INSERT ON mod_remove_post
    REFERENCING NEW TABLE AS new_removal
    FOR EACH STATEMENT
    EXECUTE FUNCTION resolve_reports_when_post_removed ();

-- These triggers create and update rows in each aggregates table to match its associated table's rows.
-- Deleting rows and updating IDs are already handled by `CASCADE` in foreign key constraints.

CREATE FUNCTION comment_aggregates_from_comment ()
    RETURNS trigger
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
    AFTER INSERT ON comment
    REFERENCING NEW TABLE AS new_comment
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.comment_aggregates_from_comment ();

CREATE FUNCTION r.community_aggregates_from_community ()
    RETURNS trigger
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
    AFTER INSERT ON community
    REFERENCING NEW TABLE AS new_community
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.community_aggregates_from_community ();

CREATE FUNCTION person_aggregates_from_person ()
    RETURNS trigger
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
    AFTER INSERT ON person
    REFERENCING NEW TABLE AS new_person
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.person_aggregates_from_person ();

CREATE FUNCTION r.post_aggregates_from_post ()
    RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO post_aggregates (post_id, published, newest_comment_time, newest_comment_time_necro, community_id, creator_id, instance_id, featured_community, featured_local)
    SELECT
        id,
        published,
        published,
        published,
        community_id,
        creator_id,
        (SELECT community.instance_id FROM community WHERE community.id = community_id LIMIT 1),
        featured_community,
        featured_local
    FROM
        new_post
    ON CONFLICT DO UPDATE SET
        featured_community = excluded.featured_community,
        featured_local = excluded.featured_local;

    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates
    AFTER INSERT OR UPDATE OF featured_community, featured_local ON post
    REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.post_aggregates_from_post ();

CREATE FUNCTION r.site_aggregates_from_site ()
    RETURNS trigger
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
        INSERT INTO
            site_aggregates (site_id)
        VALUES
            (NEW.id);

    RETURN NULL;
END
$$;

CREATE TRIGGER aggregates
    AFTER INSERT ON site
    FOR EACH ROW
    EXECUTE FUNCTION r.site_aggregates_from_site ();

COMMIT;

