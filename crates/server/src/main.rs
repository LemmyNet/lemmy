use clap::Parser;
use lemmy_server::{start_lemmy_server, CmdArgs};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  settings::SETTINGS,
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

pub extern crate rustls;

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

  rustls::crypto::ring::default_provider()
    .install_default()
    .map_err(|_e| LemmyErrorType::Unknown("Failed to install rustls crypto provider".into()))?;

  start_lemmy_server(args).await?;
  Ok(())
}
