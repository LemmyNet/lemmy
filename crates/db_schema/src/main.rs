use lemmy_utils::error::LemmyError;

/// Runs schema setup
#[tokio::main]
async fn main() -> Result<(), LemmyError> {
  crate::utils::build_db_pool().await?;

  Ok(())
}
