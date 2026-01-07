/// Very minimal wrapper around `lemmy_diesel_utils::run` to allow running migrations without
/// compiling everything.
fn main() -> anyhow::Result<()> {
  if std::env::args().len() > 1 {
    anyhow::bail!("To set parameters for running migrations, use the lemmy_server command.");
  }

  lemmy_diesel_utils::schema_setup::run(
    lemmy_diesel_utils::schema_setup::Options::default().run(),
    &std::env::var("LEMMY_DATABASE_URL")?,
  )?;

  Ok(())
}
