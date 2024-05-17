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
use lemmy_utils::error::{LemmyError, LemmyResult};
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

struct MigrationHarnessWrapper<'a> {
  conn: &'a mut PgConnection,
}

impl<'a> MigrationHarness<Pg> for MigrationHarnessWrapper<'a> {
  fn run_migration(
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

pub fn run(db_url: &str, options: Options) -> LemmyResult<()> {
  // Migrations don't support async connection, and this function doesn't need to be async
  let mut conn = PgConnection::establish(db_url).with_context(|| "Error connecting to database")?;

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

  conn.transaction::<_, LemmyError, _>(|conn| {
    let mut wrapper = MigrationHarnessWrapper { conn };

    // * Prevent other lemmy_server processes from running this transaction simultaneously by repurposing
    // the table created by `MigrationHarness::pending_migrations` as a lock target (this doesn't block
    // normal use of the table)
    // * Drop `r` schema, so migrations don't need to be made to work both with and without things in
    // it existing
    // * Disable the trigger that prevents the Diesel CLI from running migrations
    info!("Waiting for lock...");

    let enable_migrations = if options.enable_forbid_diesel_cli_trigger {
      ""
    } else {
      "SET LOCAL lemmy.enable_migrations TO 'on';"
    };

    wrapper.conn.batch_execute(&format!("LOCK __diesel_schema_migrations IN SHARE UPDATE EXCLUSIVE MODE;DROP SCHEMA IF EXISTS r CASCADE;{enable_migrations}"))?;

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
    })().map_err(|e| anyhow!("Couldn't run DB Migrations: {e}"))?;

    // Run replaceable_schema if newest migration was applied
    if !(options.revert && !options.redo_after_revert) {
      wrapper.conn
        .batch_execute(&new_sql)
        .context("Couldn't run SQL files in crates/db_schema/replaceable_schema")?;

      let num_rows_updated = update(previously_run_sql::table)
        .set(previously_run_sql::content.eq(new_sql))
        .execute(wrapper.conn)?;

      debug_assert_eq!(num_rows_updated, 1);
    }

    Ok(())
  })?;

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
    let url = SETTINGS.get_database_url();
    let mut conn = PgConnection::establish(&url)?;

    // Start with consistent state by dropping everything
    conn.batch_execute("DROP OWNED BY CURRENT_USER;")?;

    // Run and revert all migrations, ensuring there's no mistakes in any down.sql file
    run(&url, Options::default())?;
    run(&url, Options::default().revert(None))?;

    // TODO also don't drop r, and maybe just directly call the migrationharness method here
    assert!(matches!(
      run(&url, Options::default().enable_forbid_diesel_cli_trigger()),
      Err(e) if e.to_string().contains("lemmy_server")
    ));

    // Previous run shouldn't stop this one from working
    run(&url, Options::default())?;

    Ok(())
  }
}
