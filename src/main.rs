use clap::Parser;
use lemmy_server::{init_logging, start_lemmy_server, CmdArgs};
use lemmy_utils::{error::LemmyError, settings::SETTINGS};

#[tokio::main]
pub async fn main() -> Result<(), LemmyError> {
  init_logging(&SETTINGS.opentelemetry_url)?;
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
        "old_db": {
            "path": "./pictrs/old"
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
