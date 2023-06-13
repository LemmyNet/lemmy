use lemmy_server::{init_logging, start_lemmy_server};
use lemmy_utils::{error::LemmyError, settings::SETTINGS};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[actix_web::main]
pub async fn main() -> Result<(), LemmyError> {
  #[cfg(feature = "dhat-heap")]
  let _profiler = dhat::Profiler::builder().trim_backtraces(None).build();

  init_logging(&SETTINGS.opentelemetry_url)?;
  #[cfg(not(feature = "embed-pictrs"))]
  start_lemmy_server().await?;
  #[cfg(feature = "embed-pictrs")]
  {
    pict_rs::ConfigSource::memory(serde_json::json!({
        "server": {
            "address": "127.0.0.1:8080"
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
    let (lemmy, pictrs) = tokio::join!(start_lemmy_server(), pict_rs::run());
    lemmy?;
    pictrs.expect("run pictrs");
  }
  Ok(())
}
