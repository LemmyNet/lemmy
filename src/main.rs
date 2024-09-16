use clap::Parser;
use lemmy_server::{start_lemmy_server, CmdArgs};
use lemmy_utils::error::LemmyResult;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

pub extern crate rustls;

#[tokio::main]
pub async fn main() -> LemmyResult<()> {
  let filter = EnvFilter::builder()
    .with_default_directive(LevelFilter::INFO.into())
    .from_env_lossy();
  tracing_subscriber::fmt().with_env_filter(filter).init();

  let args = CmdArgs::parse();

  rustls::crypto::ring::default_provider()
    .install_default()
    .expect("Failed to install rustls crypto provider");

  start_lemmy_server(args).await?;
  Ok(())
}
