#[cfg(test)]
mod diff_check;

use crate::schema::previously_run_sql;
use anyhow::{anyhow, Context};
use diesel::{
  connection::SimpleConnection,
  migration::{Migration, MigrationSource, MigrationVersion},
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
use diesel_migrations::MigrationHarness;
use lemmy_utils::{error::LemmyResult, settings::SETTINGS};
use std::time::Instant;
use tracing::info;

// In production, include migrations in the binary
#[cfg(not(debug_assertions))]
fn get_migration_source() -> diesel_migrations::EmbeddedMigrations {
  // Using `const` here is required by the borrow checker
  const MIGRATIONS: diesel_migrations::EmbeddedMigrations = diesel_migrations::embed_migrations!();
  MIGRATIONS
}

// Avoid recompiling when migrations are changed
#[cfg(debug_assertions)]
fn get_migration_source() -> diesel_migrations::FileBasedMigrations {
  diesel_migrations::FileBasedMigrations::find_migrations_directory()
    .expect("failed to find migrations dir")
}

/// This SQL code sets up the `r` schema, which contains things that can be safely dropped and replaced
/// instead of being changed using migrations. It may not create or modify things outside of the `r` schema
/// (indicated by `r.` before the name), unless a comment says otherwise.
const REPLACEABLE_SCHEMA: &[&str] = &[
  "CREATE SCHEMA r;",
  include_str!("../replaceable_schema/utils.sql"),
  include_str!("../replaceable_schema/triggers.sql"),
];

struct MigrationHarnessWrapper<'a, 'b> {
  conn: &'a mut PgConnection,
  options: &'b Options,
}

impl<'a, 'b> MigrationHarness<Pg> for MigrationHarnessWrapper<'a, 'b> {
  fn run_migration(
    &mut self,
    migration: &dyn Migration<Pg>,
  ) -> diesel::migration::Result<MigrationVersion<'static>> {
    let name = migration.name();

    #[cfg(test)]
    if self.options.enable_diff_check {
      let before = diff_check::get_dump();
      self.conn.run_migration(migration)?;
      self.conn.revert_migration(migration)?;
      diff_check::check_dump_diff(before, &format!("migrations/{name}/down.sql"));
    }

    let start_time = Instant::now();

    let result = self.conn.run_migration(migration);

    let duration = start_time.elapsed().as_millis();
    info!("{duration}ms run {name}");

    result
  }

  fn revert_migration(
    &mut self,
    migration: &dyn Migration<Pg>,
  ) -> diesel::migration::Result<MigrationVersion<'static>> {
    if self.options.enable_diff_check {
      unimplemented!("diff check when reverting migrations");
    }

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

// TODO: remove when diesel either adds MigrationSource impl for references or changes functions to take reference
#[derive(Clone, Copy)]
struct MigrationSourceRef<T>(
  // If this was `&T`, then the derive macros would add `Clone` and `Copy` bounds for `T`
  T,
);

impl<'a, T: MigrationSource<Pg>> MigrationSource<Pg> for MigrationSourceRef<&'a T> {
  fn migrations(&self) -> diesel::migration::Result<Vec<Box<dyn Migration<Pg>>>> {
    self.0.migrations()
  }
}

#[derive(Default)]
pub struct Options {
  enable_forbid_diesel_cli_trigger: bool,
  enable_diff_check: bool,
  revert: bool,
  revert_amount: Option<u64>,
  redo_after_revert: bool,
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
    self.revert_amount = amount;
    self
  }

  pub fn redo(mut self, amount: Option<u64>) -> Self {
    self.redo_after_revert = true;
    self.revert(amount)
  }
}

// TODO return struct with field `ran_replaceable_schema`
pub fn run(options: Options) -> LemmyResult<()> {
  let db_url = SETTINGS.get_database_url();

  // Migrations don't support async connection, and this function doesn't need to be async
  let mut conn =
    PgConnection::establish(&db_url).with_context(|| "Error connecting to database")?;

  let new_sql = REPLACEABLE_SCHEMA.join("\n");

  let migration_source = get_migration_source();
  let migration_source_ref = MigrationSourceRef(&migration_source);

  // If possible, skip locking the migrations table and recreating the "r" schema, so
  // lemmy_server processes in a horizontally scaled setup can start without causing locks
  if !(options.revert
    || conn
      .has_pending_migration(migration_source_ref)
      .map_err(|e| anyhow!("Couldn't check pending migrations: {e}"))?)
  {
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

  // Disable the trigger that prevents the Diesel CLI from running migrations
  if !options.enable_forbid_diesel_cli_trigger {
    conn.batch_execute("SET lemmy.enable_migrations TO 'on';")?;
  }

  // Running without transaction allows pg_dump to see results of migrations
  let run_in_transaction = !options.enable_diff_check;

  let transaction = |conn: &mut PgConnection| -> LemmyResult<()> {
    let mut wrapper = MigrationHarnessWrapper {
      conn,
      options: &options,
    };

    // * Prevent other lemmy_server processes from running this transaction simultaneously by repurposing
    // the table created by `MigrationHarness::pending_migrations` as a lock target (this doesn't block
    // normal use of the table)
    // * Drop `r` schema, so migrations don't need to be made to work both with and without things in
    // it existing
    info!("Waiting for lock...");

    let lock = if run_in_transaction {
      "LOCK __diesel_schema_migrations IN SHARE UPDATE EXCLUSIVE MODE;"
    } else {
      ""
    };

    wrapper
      .conn
      .batch_execute(&format!("{lock}DROP SCHEMA IF EXISTS r CASCADE;"))?;

    info!("Running Database migrations (This may take a long time)...");

    (|| {
      if options.revert {
        if let Some(amount) = options.revert_amount {
          for _ in 0..amount {
            wrapper.revert_last_migration(migration_source_ref)?;
          }
          if options.redo_after_revert {
            for _ in 0..amount {
              wrapper.run_next_migration(migration_source_ref)?;
            }
          }
        } else {
          wrapper.revert_all_migrations(migration_source_ref)?;
          if options.redo_after_revert {
            wrapper.run_pending_migrations(migration_source_ref)?;
          }
        }
      } else {
        wrapper.run_pending_migrations(migration_source_ref)?;
      }
      diesel::migration::Result::Ok(())
    })()
    .map_err(|e| anyhow!("Couldn't run DB Migrations: {e}"))?;

    // Run replaceable_schema if newest migration was applied
    if !(options.revert && !options.redo_after_revert) {
      #[cfg(test)]
      if options.enable_diff_check {
        let before = diff_check::get_dump();
        // todo move replaceable_schema dir path to let/const?
        wrapper
          .conn
          .batch_execute(&new_sql)
          .context("Couldn't run SQL files in crates/db_schema/replaceable_schema")?;
        // todo move statement to const
        wrapper
          .conn
          .batch_execute("DROP SCHEMA IF EXISTS r CASCADE;")?;
        diff_check::check_dump_diff(before, "replaceable_schema");
      }

      wrapper
        .conn
        .batch_execute(&new_sql)
        .context("Couldn't run SQL files in crates/db_schema/replaceable_schema")?;

      let num_rows_updated = update(previously_run_sql::table)
        .set(previously_run_sql::content.eq(new_sql))
        .execute(wrapper.conn)?;

      debug_assert_eq!(num_rows_updated, 1);
    }

    Ok(())
  };

  if run_in_transaction {
    conn.transaction(transaction)?;
  } else {
    transaction(&mut conn)?;
  }

  info!("Database migrations complete.");

  Ok(())
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
