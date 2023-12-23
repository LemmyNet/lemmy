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

-- These triggers create and update rows in each aggregates table to match its associated table's rows.
-- Deleting rows and updating IDs are already handled by `CASCADE` in foreign key constraints.

CREATE FUNCTION r.community_aggregates_from_community () RETURNS trigger
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

CREATE FUNCTION r.post_aggregates_from_post () RETURNS trigger
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

COMMIT;

