-- This trigger prevents using the Diesel CLI to run or revert migrations, so the custom migration runner
-- can drop and recreate the `r` schema for new migrations.
--
-- This migration being seperate from the next migration (created in the same PR) guarantees that the
-- Diesel CLI will fail to bring the number of pending migrations to 0, which is one of the conditions
-- required to skip running replaceable_schema.
--
-- If the Diesel CLI could run or revert migrations, this scenario would be possible:
--
-- Run `diesel migration redo` when the newest migration has a new table with triggers. End up with triggers
-- being dropped and not replaced because triggers are created outside of up.sql. The custom migration runner
-- sees that there are no pending migrations and the value in the `previously_run_sql` trigger is correct, so
-- it doesn't rebuild the `r` schema. There is now incorrect behavior but no error messages.
CREATE FUNCTION forbid_diesel_cli ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NOT EXISTS (
        SELECT
        FROM
            pg_locks
        WHERE (locktype, pid, objid) = ('advisory', pg_backend_pid(), 0)) THEN
RAISE 'migrations must be managed using lemmy_server instead of diesel CLI';
END IF;
    RETURN NULL;
END;
$$;

CREATE TRIGGER forbid_diesel_cli
    BEFORE INSERT OR UPDATE OR DELETE OR TRUNCATE ON __diesel_schema_migrations
    EXECUTE FUNCTION forbid_diesel_cli ();

