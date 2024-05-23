use clap::Parser;
use lemmy_server::{start_lemmy_server, CmdArgs};
use lemmy_utils::error::LemmyResult;
use tracing::{level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[tokio::main]
pub async fn main() -> LemmyResult<()> {
  let filter = EnvFilter::builder()
  .with_default_directive(LevelFilter::INFO.into())
  .from_env_lossy();
  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .init();

  let args = CmdArgs::parse();

  #[cfg(not(feature = "embed-pictrs"))]
  start_lemmy_server(args).await?;
  #[cfg(feature = "embed-pictrs")]
  {
    let pictrs_port = &SETTINGS
      .pictrs_config()
      .unwrap_or_default()
      .url
      .port()
      .unwrap_or(8080);
    let pictrs_address = ["127.0.0.1", &pictrs_port.to_string()].join(":");
    let pictrs_config = pict_rs::ConfigSource::memory(serde_json::json!({
        "server": {
            "address": pictrs_address
        },
        "repo": {
            "type": "sled",
            "path": "./pictrs/sled-repo"
        },
        "store": {
            "type": "filesystem",
            "path": "./pictrs/files"
        }
    }))
    .init::<&str>(None)
    .expect("initialize pictrs config");
    let (lemmy, pictrs) = tokio::join!(start_lemmy_server(args), pictrs_config.run_on_localset());
    lemmy?;
    pictrs.expect("run pictrs");
  }
  Ok(())
}
