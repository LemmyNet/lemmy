use crate::{schema::previously_run_sql};
use anyhow::Context;
use diesel::{
  connection::SimpleConnection,
  select,
  update,
  Connection,
  ExpressionMethods,
  PgConnection,
  QueryDsl,
  RunQueryDsl,NullableExpressionMethods
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use lemmy_utils::error::LemmyError;
use std::time::Instant;
use tracing::info;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// This SQL code sets up the `r` schema, which contains things that can be safely dropped and replaced
/// instead of being changed using migrations. It may not create or modify things outside of the `r` schema
/// (indicated by `r.` before the name), unless a comment says otherwise.
const REPLACEABLE_SCHEMA: &[&str] = &[
  "CREATE SCHEMA r;",
  include_str!("../replaceable_schema/utils.sql"),
  include_str!("../replaceable_schema/triggers.sql"),
];

const REVERT_REPLACEABLE_SCHEMA: &str = "DROP SCHEMA IF EXISTS r CASCADE;";

// TODO use full names
const FORBID_DIESEL_CLI_MIGRATION_VERSION: &str = "0000000000000";

const CUSTOM_MIGRATION_RUNNER_MIGRATION_VERSION: &str = "2024-04-29-012113";

pub fn run(db_url: &str) -> Result<(), LemmyError> {
  // Migrations don't support async connection
  let mut conn = PgConnection::establish(db_url).with_context(|| "Error connecting to database")?;

  let test_enabled = std::env::var("LEMMY_TEST_MIGRATIONS")
    .map(|s| !s.is_empty())
    .unwrap_or(false);

  let new_sql = REPLACEABLE_SCHEMA.join("\n");

  // Early return should be as fast as possible and not do any locks in the database, because this case
  // is reached whenever a lemmy_server process is started, which can happen frequently on a production server
  // with a horizontally scaled setup.
  let unfiltered_pending_migrations = conn
    .pending_migrations(MIGRATIONS)
    .map_err(|e| anyhow::anyhow!("Couldn't determine pending migrations: {e}"))?;

  // Check len first so this doesn't run without the previously_run_sql table existing
  if unfiltered_pending_migrations.len() == 1 {
    let sql_unchanged: bool = select(
      previously_run_sql::table
        .select(previously_run_sql::content)
        .single_value()
        .assume_not_null()
        .eq(&new_sql),
    )
    .get_result(&mut conn)?;

    if sql_unchanged {
      debug_assert_eq!(
        unfiltered_pending_migrations
        .get(0)
        .map(|m| m.name().version()),
        Some(FORBID_DIESEL_CLI_MIGRATION_VERSION.into())
      );
      return Ok(());
    }
  }

  conn.transaction::<_, LemmyError, _>(|conn| {
    // Use the table created by `MigrationHarness::pending_migrations` as a lock target to prevent multiple
    // lemmy_server processes from running this transaction concurrently. This lock does not block
    // `MigrationHarness::pending_migrations` (`SELECT`) or `MigrationHarness::run_migration` (`INSERT`).
    info!("Waiting for lock...");
    conn.batch_execute("LOCK __diesel_schema_migrations IN SHARE UPDATE EXCLUSIVE MODE;")?;
    info!("Running Database migrations (This may take a long time)...");
    // Check pending migrations again after locking
    let unfiltered_pending_migrations = conn.pending_migrations(MIGRATIONS).map_err(|e| anyhow::anyhow!("Couldn't determine pending migrations: {e}"))?;

    // Does not include the "forbid_diesel_cli" migration
    let pending_migrations = unfiltered_pending_migrations.get(1..).expect(
      "original pending migrations length should be at least 1 because of the forbid_diesel_cli migration",
    );

    // Check migration version constants in debug mode
    debug_assert_eq!(
      unfiltered_pending_migrations
      .get(0)
      .map(|m| m.name().version()),
      Some(FORBID_DIESEL_CLI_MIGRATION_VERSION.into())
    );
    debug_assert_eq!(
      pending_migrations
      .iter()
      .filter(|m| m.name().version() == FORBID_DIESEL_CLI_MIGRATION_VERSION.into())
      .count(),
      0
    );
    /*TODO maybe do this for all migrations not just pending
    debug_assert_eq!(
      pending_migrations
      .iter()
      .filter(|m| m.name().version() == CUSTOM_MIGRATION_RUNNER_MIGRATION_VERSION.into())
      .count(),
      1
    );*/

    // Run migrations, without stuff from replaceable_schema
    conn.batch_execute(REVERT_REPLACEABLE_SCHEMA).context("Couldn't drop schema `r`")?;
    for migration in pending_migrations {
      let name = migration.name();
      // TODO measure time on database
      let start_time = Instant::now();
      conn.run_migration(migration)
      .map_err(|e| anyhow::anyhow!("Couldn't run migration {name}: {e}"))?;
    let duration = start_time.elapsed().as_millis();
      info!("{duration}ms {name}");
    }

    // Run replaceable_schema
    conn.batch_execute(&new_sql)
    .context("Couldn't run SQL files in crates/db_schema/replaceable_schema")?;

  let num_rows_updated = update(previously_run_sql::table)
    .set(previously_run_sql::content.eq(new_sql))
    .execute(conn)?;

  debug_assert_eq!(num_rows_updated, 1);

  Ok(())
})?;
  info!("Database migrations complete.");

  Ok(())
}
