/// Very minimal wrapper around `lemmy_db_schema_setup::run` to allow running migrations without
/// compiling everything.
fn main() -> anyhow::Result<()> {
  if std::env::args().len() > 1 {
    anyhow::bail!("To set parameters for running migrations, use the lemmy_server command.");
  }

  lemmy_db_schema_setup::run(
    lemmy_db_schema_setup::Options::default().run(),
    &std::env::var("LEMMY_DATABASE_URL")?,
  )?;

  Ok(())
}
