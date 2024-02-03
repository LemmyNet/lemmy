use diesel_migrations::MigrationHarness;
use anyhow::Context;
use diesel::{Connection, connection::SimpleConnection};
use std::path::Path;
use diesel::PgConnection;
use diesel_migrations::EmbeddedMigrations;
use lemmy_utils::error::LemmyError;
use tracing::info;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();


/// This SQL code sets up the `r` schema, which contains things that can be safely dropped and replaced
/// instead of being changed using migrations. It may not create or modify things outside of the `r` schema
/// (indicated by `r.` before the name), unless a comment says otherwise.
///
/// If you add something that depends on something (such as a table) created in a new migration, then down.sql
/// must use `CASCADE` when dropping it. This doesn't need to be fixed in old migrations because the
/// "replaceable-schema" migration runs `DROP SCHEMA IF EXISTS r CASCADE` in down.sql.
const REPLACEABLE_SCHEMA: &[&str] = &[
  "BEGIN;",
  "DROP SCHEMA IF EXISTS r CASCADE;",
  "CREATE SCHEMA r;",
  //include_str!("triggers.sql"),
  include_str!("../../../../replaceable_schema.sql"),
  "COMMIT;",
];

pub fn run(db_url: &str) -> Result<(), LemmyError> {
  // Migrations don't support async connection
  let mut conn =
    PgConnection::establish(db_url).with_context(|| format!("Error connecting to {db_url}"))?;

  // Migrations
  info!("Running Database migrations (This may take a long time)...");
  conn
    .run_pending_migrations(MIGRATIONS)
    .map_err(|e| anyhow::anyhow!("Couldn't run DB Migrations: {e}"))?;
  info!("Database migrations complete.");

  // Replaceable schema
  conn
    .batch_execute(&REPLACEABLE_SCHEMA.join("\n"))
    .with_context(|| format!("Couldn't run SQL files in {}", Path::new(file!()).parent().map(|p| p.to_string_lossy()).unwrap_or("".into())))?;

  Ok(())
}
