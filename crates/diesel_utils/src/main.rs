use diesel_migrations::MigrationHarness;

/// Very minimal wrapper to allow running migrations without
/// compiling everything.
fn main() -> anyhow::Result<()> {
  if std::env::args().len() > 1 {
    anyhow::bail!("To set parameters for running migrations, use the lemmy_server command.");
  }

  // todo: set the application_name
  let mut harness =
    lemmy_diesel_utils::schema_setup::Options::new(&std::env::var("LEMMY_DATABASE_URL")?)?;
  harness
    .run_pending_migrations(lemmy_diesel_utils::schema_setup::migrations())
    .map_err(lemmy_diesel_utils::schema_setup::convert_err)?;
  lemmy_diesel_utils::schema_setup::run_replaceable_schema(&mut harness.conn)?;

  Ok(())
}
