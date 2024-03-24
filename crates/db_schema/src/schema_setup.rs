use anyhow::Context;
use diesel::{connection::SimpleConnection, Connection, PgConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use lemmy_utils::error::LemmyError;
use tracing::info;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// This SQL code sets up the `r` schema, which contains things that can be safely dropped and replaced
/// instead of being changed using migrations. It may not create or modify things outside of the `r` schema
/// (indicated by `r.` before the name), unless a comment says otherwise.
///
/// Currently, this code is only run after the server starts and there's at least 1 pending migration
/// to run. This means every time you change something here, you must also create a migration (a blank
/// up.sql file works fine). This behavior will be removed when we implement a better way to avoid
/// useless schema updates and locks.
///
/// If you add something that depends on something (such as a table) created in a new migration, then down.sql
/// must use `CASCADE` when dropping it. This doesn't need to be fixed in old migrations because the
/// "replaceable-schema" migration runs `DROP SCHEMA IF EXISTS r CASCADE` in down.sql.
const REPLACEABLE_SCHEMA: &[&str] = &[
  "DROP SCHEMA IF EXISTS r CASCADE;",
  "CREATE SCHEMA r;",
  include_str!("../replaceable_schema/utils.sql"),
  include_str!("../replaceable_schema/triggers.sql"),
];

pub fn run(db_url: &str) -> Result<(), LemmyError> {
  // Migrations don't support async connection
  let mut conn = PgConnection::establish(db_url).with_context(|| "Error connecting to database")?;

  // Run all pending migrations except for the newest one, then run the newest one in the same transaction
  // as `REPLACEABLE_SCHEMA`. This code will be becone less hacky when the conditional setup of things in
  // `REPLACEABLE_SCHEMA` is done without using the number of pending migrations.
  info!("Running Database migrations (This may take a long time)...");
  let migrations = conn.pending_migrations(MIGRATIONS)?;
  for migration in migrations.iter().rev().skip(1).rev() {
    conn
      .run_migration(&migration)
      .map_err(|e| anyhow::anyhow!("Couldn't run DB Migrations: {e}"))?;
  }
  conn.transaction::<_, LemmyError, _>(|conn| {
    if let Some(migration) = migrations.last() {
      // Migration is run with a savepoint since there's already a transaction
      conn
        .run_migration(&migration)
        .map_err(|e| anyhow::anyhow!("Couldn't run DB Migrations: {e}"))?;
    } else if !dbg(debug_assertions) {
      // In production, skip running `REPLACEABLE_SCHEMA` to avoid locking things in the schema. In
      // CI, always run it because `diesel migration` commands would otherwise prevent it.
      return Ok(());
    }
    conn
      .batch_execute(&REPLACEABLE_SCHEMA.join("\n"))
      .context("Couldn't run SQL files in crates/db_schema/replaceable_schema")?;

    Ok(())
  })?;
  info!("Database migrations complete.");

  Ok(())
}
