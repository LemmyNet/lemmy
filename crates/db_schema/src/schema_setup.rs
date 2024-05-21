#[cfg(test)]
mod diff_check;

use crate::schema::previously_run_sql;
use anyhow::{anyhow, Context};
use diesel::{
  connection::SimpleConnection,
  dsl::exists,
  expression::IntoSql,
  migration::{Migration, MigrationSource, MigrationVersion},
  pg::Pg,
  select,
  sql_types,
  update,
  BoolExpressionMethods,
  Connection,
  ExpressionMethods,
  NullableExpressionMethods,
  PgConnection,
  QueryDsl,
  RunQueryDsl,
};
use diesel_migrations::MigrationHarness;
use lemmy_utils::{error::LemmyResult, settings::SETTINGS};
use std::time::Instant;
use tracing::info;

diesel::table! {
  pg_namespace (nspname) {
    nspname -> Text,
  }
}

// In production, include migrations in the binary
#[cfg(not(debug_assertions))]
fn migrations() -> diesel_migrations::EmbeddedMigrations {
  // Using `const` here is required by the borrow checker
  const MIGRATIONS: diesel_migrations::EmbeddedMigrations = diesel_migrations::embed_migrations!();
  MIGRATIONS
}

// Avoid recompiling when migrations are changed
#[cfg(debug_assertions)]
fn migrations() -> diesel_migrations::FileBasedMigrations {
  diesel_migrations::FileBasedMigrations::find_migrations_directory()
    .expect("failed to get migration source")
}

/// This SQL code sets up the `r` schema, which contains things that can be safely dropped and replaced
/// instead of being changed using migrations. It may not create or modify things outside of the `r` schema
/// (indicated by `r.` before the name), unless a comment says otherwise.
fn replaceable_schema() -> String {
  [
    "CREATE SCHEMA r;",
    include_str!("../replaceable_schema/utils.sql"),
    include_str!("../replaceable_schema/triggers.sql"),
  ]
  .join("\n")
}

const REPLACEABLE_SCHEMA_PATH: &str = "crates/db_schema/replaceable_schema";

struct MigrationHarnessWrapper<'a, 'b> {
  conn: &'a mut PgConnection,
  options: &'b Options,
}

impl<'a, 'b> MigrationHarnessWrapper<'a, 'b> {
  fn run_migration_inner(
    &mut self,
    migration: &dyn Migration<Pg>,
  ) -> diesel::migration::Result<MigrationVersion<'static>> {
    let start_time = Instant::now();

    let result = self.conn.run_migration(migration);

    let duration = start_time.elapsed().as_millis();
    let name = migration.name();
    info!("{duration}ms run {name}");

    result
  }
}

impl<'a, 'b> MigrationHarness<Pg> for MigrationHarnessWrapper<'a, 'b> {
  fn run_migration(
    &mut self,
    migration: &dyn Migration<Pg>,
  ) -> diesel::migration::Result<MigrationVersion<'static>> {
    #[cfg(test)]
    if self.options.enable_diff_check {
      let before = diff_check::get_dump();

      self.run_migration_inner(migration)?;
      self.revert_migration(migration)?;

      let after = diff_check::get_dump();

      diff_check::check_dump_diff(
        after,
        before,
        &format!(
          "These changes need to be applied in migrations/{}/down.sql:",
          migration.name()
        ),
      );
    }

    self.run_migration_inner(migration)
  }

  fn revert_migration(
    &mut self,
    migration: &dyn Migration<Pg>,
  ) -> diesel::migration::Result<MigrationVersion<'static>> {
    let start_time = Instant::now();

    let result = self.conn.revert_migration(migration);

    let duration = start_time.elapsed().as_millis();
    let name = migration.name();
    info!("{duration}ms revert {name}");

    result
  }

  fn applied_migrations(&mut self) -> diesel::migration::Result<Vec<MigrationVersion<'static>>> {
    self.conn.applied_migrations()
  }
}

pub struct Options {
  enable_forbid_diesel_cli_trigger: bool,
  enable_diff_check: bool,
  revert: bool,
  run: bool,
  amount: Option<u64>,
}

impl Default for Options {
  fn default() -> Self {
    Options {
      enable_forbid_diesel_cli_trigger: false,
      enable_diff_check: false,
      revert: false,
      run: true,
      amount: None,
    }
  }
}

impl Options {
  #[cfg(test)]
  fn enable_forbid_diesel_cli_trigger(mut self) -> Self {
    self.enable_forbid_diesel_cli_trigger = true;
    self
  }

  #[cfg(test)]
  fn enable_diff_check(mut self) -> Self {
    self.enable_diff_check = true;
    self
  }

  pub fn revert(mut self, amount: Option<u64>) -> Self {
    self.revert = true;
    self.run = false;
    self.amount = amount;
    self
  }

  pub fn redo(mut self, amount: Option<u64>) -> Self {
    self.revert = true;
    self.run = true;
    self.amount = amount;
    self
  }
}

// TODO return struct with field `ran_replaceable_schema`
pub fn run(options: Options) -> LemmyResult<()> {
  let db_url = SETTINGS.get_database_url();

  // Migrations don't support async connection, and this function doesn't need to be async
  //let mut conn = PgConnection::establish(&db_url).context("Error connecting to database")?;
  let mut conn = PgConnection::establish(&db_url)?;

  // If possible, skip locking the migrations table and recreating the "r" schema, so
  // lemmy_server processes in a horizontally scaled setup can start without causing locks
  if !options.revert
    && options.run
    && options.amount.is_none()
    && !conn
      .has_pending_migration(migrations())
      .map_err(convert_err)?
  //.map_err(|e| anyhow!("Couldn't check pending migrations: {e}"))?)
  {
    // The condition above implies that the migration that creates the previously_run_sql table was already run
    let sql_unchanged = exists(
      previously_run_sql::table.filter(previously_run_sql::content.eq(replaceable_schema())),
    );

    let schema_exists = exists(pg_namespace::table.find("r"));

    if select(sql_unchanged.and(schema_exists)).get_result(&mut conn)? {
      return Ok(());
    }
  }

  // Disable the trigger that prevents the Diesel CLI from running migrations
  if !options.enable_forbid_diesel_cli_trigger {
    conn.batch_execute("SET lemmy.enable_migrations TO 'on';")?;
  }

  // Block concurrent attempts to run migrations (lock is automatically released at the end
  // because the connection is not reused)
  info!("Waiting for lock...");
  conn.batch_execute("SELECT pg_advisory_lock(0)")?;
  info!("Running Database migrations (This may take a long time)...");

  // Drop `r` schema, so migrations don't need to be made to work both with and without things in
  // it existing
  revert_replaceable_schema(&mut conn)?;

  run_selected_migrations(&mut conn, &options).map_err(convert_err)?;

  // Only run replaceable_schema if newest migration was applied
  if (options.run && options.amount.is_none())
    || !conn
      .has_pending_migration(migrations())
      .map_err(convert_err)?
  {
    #[cfg(test)]
    if options.enable_diff_check {
      let before = diff_check::get_dump();

      run_replaceable_schema(&mut conn)?;
      revert_replaceable_schema(&mut conn)?;

      let after = diff_check::get_dump();

      diff_check::check_dump_diff(before, after, "The code in crates/db_schema/replaceable_schema incorrectly created or modified things outside of the `r` schema, causing these changes to be left behind after dropping the schema:");
    }

    run_replaceable_schema(&mut conn)?;
  }

  info!("Database migrations complete.");

  Ok(())
}

fn run_replaceable_schema(conn: &mut PgConnection) -> LemmyResult<()> {
  conn.transaction(|conn| {
    conn
      .batch_execute(&replaceable_schema())
      .with_context(|| format!("Failed to run SQL files in {REPLACEABLE_SCHEMA_PATH}"))?;

    let num_rows_updated = update(previously_run_sql::table)
      .set(previously_run_sql::content.eq(replaceable_schema()))
      .execute(conn)?;

    debug_assert_eq!(num_rows_updated, 1);

    Ok(())
  })
}

fn revert_replaceable_schema(conn: &mut PgConnection) -> LemmyResult<()> {
  conn
    .batch_execute("DROP SCHEMA IF EXISTS r CASCADE;")
    .with_context(|| format!("Failed to revert SQL files in {REPLACEABLE_SCHEMA_PATH}"))?;

  // Value in `previously_run_sql` table is not set here because the table might not exist,
  // and that's fine because the existence of the `r` schema is also checked

  Ok(())
}

fn run_selected_migrations(
  conn: &mut PgConnection,
  options: &Options,
) -> diesel::migration::Result<()> {
  let mut wrapper = MigrationHarnessWrapper { conn, options };

  if options.revert {
    if let Some(amount) = options.amount {
      for _ in 0..amount {
        wrapper.revert_last_migration(migrations())?;
      }
    } else {
      wrapper.revert_all_migrations(migrations())?;
    }
  }

  if options.run {
    if let Some(amount) = options.amount {
      for _ in 0..amount {
        wrapper.run_next_migration(migrations())?;
      }
    } else {
      wrapper.run_pending_migrations(migrations())?;
    }
  }

  Ok(())
}

/// Makes `diesel::migration::Result` work with `anyhow` and `LemmyError`
fn convert_err(
  err: Box<dyn std::error::Error + Send + Sync>,
  //) -> impl std::error::Error + Send + Sync + 'static {
) -> anyhow::Error {
  anyhow::anyhow!(err)
}

#[cfg(test)]
mod tests {
  use super::*;
  use lemmy_utils::{error::LemmyResult, settings::SETTINGS};
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_schema_setup() -> LemmyResult<()> {
    let db_url = SETTINGS.get_database_url();
    let mut conn = PgConnection::establish(&db_url)?;

    // Start with consistent state by dropping everything
    conn.batch_execute("DROP OWNED BY CURRENT_USER;")?;

    // Check for mistakes in down.sql files
    run(Options::default().enable_diff_check())?;

    // TODO also don't drop r, and maybe just directly call the migrationharness method here
    run(Options::default().revert(None))?;
    assert!(matches!(
      run(Options::default().enable_forbid_diesel_cli_trigger()),
      Err(e) if e.to_string().contains("lemmy_server")
    ));

    // Previous run shouldn't stop this one from working
    run(Options::default())?;

    Ok(())
  }
}
