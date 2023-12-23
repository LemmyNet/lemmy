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

CREATE FUNCTION r.upsert_community_aggregates_from_community () RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO
        community_aggregates (community_id, published)
    SELECT
        community_id,
        published
    FROM
        new_community
    ON CONFLICT DO UPDATE SET
        published = excluded.published;

    RETURN NULL;
END
$$;

CREATE TRIGGER upsert_aggregates
    AFTER INSERT OR UPDATE OF published ON community
    REFERENCING NEW TABLE AS new_community
    FOR EACH STATEMENT
    EXECUTE FUNCTION r.upsert_community_aggregates_from_community;

COMMIT;

