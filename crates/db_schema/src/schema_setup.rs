use crate::schema::previously_run_sql;
use anyhow::Context;
use diesel::{
  backend::Backend,
  connection::SimpleConnection,
  migration::{Migration, MigrationSource},
  pg::Pg,
  select,
  update,
  Connection,
  ExpressionMethods,
  NullableExpressionMethods,
  PgConnection,
  QueryDsl,
  RunQueryDsl,
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use lemmy_utils::error::{LemmyError, LemmyResult};
use std::time::Instant;
use tracing::info;

const EMBEDDED_MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// This SQL code sets up the `r` schema, which contains things that can be safely dropped and replaced
/// instead of being changed using migrations. It may not create or modify things outside of the `r` schema
/// (indicated by `r.` before the name), unless a comment says otherwise.
const REPLACEABLE_SCHEMA: &[&str] = &[
  "CREATE SCHEMA r;",
  include_str!("../replaceable_schema/utils.sql"),
  include_str!("../replaceable_schema/triggers.sql"),
];

const REVERT_REPLACEABLE_SCHEMA: &str = "DROP SCHEMA IF EXISTS r CASCADE;";

const LOCK_STATEMENT: &str = "LOCK __diesel_schema_migrations IN SHARE UPDATE EXCLUSIVE MODE;";

struct Migrations;

impl<DB: Backend> MigrationSource<DB> for Migrations {
  fn migrations(&self) -> diesel::migration::Result<Vec<Box<dyn Migration<DB>>>> {
    let mut migrations = EMBEDDED_MIGRATIONS.migrations()?;
    let skipped_migration = if migrations.is_empty() {
      None
    } else {
      Some(migrations.remove(0))
    };

    debug_assert_eq!(
      skipped_migration.map(|m| m.name().to_string()),
      Some("000000000000000_forbid_diesel_cli".to_string())
    );

    Ok(migrations)
  }
}

fn get_pending_migrations(conn: &mut PgConnection) -> LemmyResult<Vec<Box<dyn Migration<Pg>>>> {
  Ok(
    conn
      .pending_migrations(Migrations)
      .map_err(|e| anyhow::anyhow!("Couldn't determine pending migrations: {e}"))?,
  )
}

pub fn run(db_url: &str) -> LemmyResult<()> {
  // Migrations don't support async connection
  let mut conn = PgConnection::establish(db_url).with_context(|| "Error connecting to database")?;

  let test_enabled = std::env::var("LEMMY_TEST_MIGRATIONS")
    .map(|s| !s.is_empty())
    .unwrap_or(false);

  let new_sql = REPLACEABLE_SCHEMA.join("\n");

  let pending_migrations = get_pending_migrations(&mut conn)?;

  // If possible, skip locking the migrations table and recreating the "r" schema, so
  // lemmy_server processes in a horizontally scaled setup can start without causing locks
  if pending_migrations.is_empty() {
    // The condition above implies that the migration that creates the previously_run_sql table was already run
    let sql_unchanged: bool = select(
      previously_run_sql::table
        .select(previously_run_sql::content)
        .single_value()
        .assume_not_null()
        .eq(&new_sql),
    )
    .get_result(&mut conn)?;

    if sql_unchanged {
      return Ok(());
    }
  }

  conn.transaction::<_, LemmyError, _>(|conn| {
    // Use the table created by `MigrationHarness::pending_migrations` as a lock target to prevent multiple
    // lemmy_server processes from running this transaction concurrently. This lock does not block
    // `MigrationHarness::pending_migrations` (`SELECT`) or `MigrationHarness::run_migration` (`INSERT`).
    info!("Waiting for lock...");
    conn.batch_execute(LOCK_STATEMENT)?;
    info!("Running Database migrations (This may take a long time)...");

    // Check pending migrations again after locking
    let pending_migrations = get_pending_migrations(conn)?;

    // Run migrations, without stuff from replaceable_schema
    conn.batch_execute(REVERT_REPLACEABLE_SCHEMA)?;

    for migration in &pending_migrations {
      let name = migration.name();
      let start_time = Instant::now();
      conn
        .run_migration(migration)
        .map_err(|e| anyhow::anyhow!("Couldn't run migration {name}: {e}"))?;
      let duration = start_time.elapsed().as_millis();
      info!("{duration}ms run {name}");
    }

    // Run replaceable_schema
    conn
      .batch_execute(&new_sql)
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
