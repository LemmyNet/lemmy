use lemmy_utils::error::LemmyError;

/// Runs schema setup
#[tokio::main]
async fn main() -> Result<(), LemmyError> {
  lemmy_db_schema::utils::build_db_pool().await?;

  Ok(())
}
