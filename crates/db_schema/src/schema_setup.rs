use anyhow::Context;
use diesel::{connection::SimpleConnection, Connection, PgConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use lemmy_utils::error::LemmyError;
use tracing::info;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// This SQL code sets up the `r` schema, which contains things that can be safely dropped and replaced
/// instead of being changed using migrations. It may not create or modify things outside of the `r` schema
/// (indicated by `r.` before the name), unless a comment says otherwise.
const REPLACEABLE_SCHEMA: &[&str] = &[
  "DROP SCHEMA IF EXISTS r CASCADE;",
  "CREATE SCHEMA r;",
  include_str!("../replaceable_schema/utils.sql"),
  include_str!("../replaceable_schema/triggers.sql"),
];

pub fn run(db_url: &str) -> Result<(), LemmyError> {
  let test_enabled = std::env::var("LEMMY_TEST_MIGRATIONS")
    .map(|s| !s.is_empty())
    .unwrap_or(false);

  // Migrations don't support async connection
  let mut conn = PgConnection::establish(db_url).with_context(|| "Error connecting to database")?;

  info!("Running Database migrations (This may take a long time)...");

  let unfiltered_migrations = conn
    .pending_migrations(MIGRATIONS)
    .map_err(|e| anyhow::anyhow!("Couldn't determine pending migrations: {e}"))?;

  // Does not include the "forbid_diesel_cli" migration
  let migrations = unfiltered_migrations.iter().filter(|m| m.name().version() != "000000000000000".into());

  conn.transaction::<_, LemmyError, _>(|conn|) // left off here

  for migration in migrations.clone() {
    conn
      .run_migration(migration)
      .map_err(|e| anyhow::anyhow!("Couldn't run DB Migrations: {e}"))?;
  }
  conn.transaction::<_, LemmyError, _>(|conn| {
    if let Some(migration) = migrations.last() {
      // Migration is run with a savepoint since there's already a transaction
      conn
        .run_migration(migration)
        .map_err(|e| anyhow::anyhow!("Couldn't run DB Migrations: {e}"))?;
    } else if !cfg!(debug_assertions) {
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
