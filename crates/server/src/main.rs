use clap::Parser;
use lemmy_server::{CmdArgs, start_lemmy_server};
use lemmy_utils::{error::LemmyResult, settings::SETTINGS};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[tokio::main]
pub async fn main() -> LemmyResult<()> {
  let filter = EnvFilter::builder()
    .with_default_directive(LevelFilter::INFO.into())
    .from_env_lossy();
  if SETTINGS.json_logging {
    tracing_subscriber::fmt()
      .with_env_filter(filter)
      .json()
      .init();
  } else {
    tracing_subscriber::fmt().with_env_filter(filter).init();
  }

  let args = CmdArgs::parse();

  start_lemmy_server(args).await?;
  Ok(())
}
