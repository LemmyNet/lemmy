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

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// This SQL code sets up the `r` schema, which contains things that can be safely dropped and replaced
/// instead of being changed using migrations. It may not create or modify things outside of the `r` schema
/// (indicated by `r.` before the name), unless a comment says otherwise.
const REPLACEABLE_SCHEMA: &[&str] = &[
  "CREATE SCHEMA r;",
  include_str!("../replaceable_schema/utils.sql"),
  include_str!("../replaceable_schema/triggers.sql"),
];

fn get_pending_migrations(conn: &mut PgConnection) -> LemmyResult<Vec<Box<dyn Migration<Pg>>>> {
  Ok(
    conn
      .pending_migrations(MIGRATIONS)
      .map_err(|e| anyhow::anyhow!("Couldn't determine pending migrations: {e}"))?,
  )
}

#[derive(Default)]
pub struct Options {
  /// Only for testing
  disable_migrations: bool,
}

pub fn run(db_url: &str, options: &Options) -> LemmyResult<()> {
  // Migrations don't support async connection, and this function doesn't need to be async
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
    conn.batch_execute("LOCK __diesel_schema_migrations IN SHARE UPDATE EXCLUSIVE MODE;")?;
    info!("Running Database migrations (This may take a long time)...");

    // Check pending migrations again after locking
    let pending_migrations = get_pending_migrations(conn)?;

    // Drop `r` schema and disable the trigger that prevents the Diesel CLI from running migrations
    let enable_migrations = if options.disable_migrations {
      ""
    } else {
      "SET LOCAL lemmy.enable_migrations TO 'on';"
    };
    conn.batch_execute(&format!(
      "DROP SCHEMA IF EXISTS r CASCADE;{enable_migrations}"
    ))?;

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

#[cfg(test)]
mod tests {
  use lemmy_utils::{error::LemmyResult, settings::SETTINGS};

  #[test]
  fn test_schema_setup() -> LemmyResult<()> {
    let mut options = super::Options::default();
    let db_url = SETTINGS.get_database_url();

    // Test the forbid_diesel_cli trigger
    options.disable_migrations = true;
    super::run(&db_url, &options).expect_err("forbid_diesel_cli trigger should throw error");

    Ok(())
  }
}
