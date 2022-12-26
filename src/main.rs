use lemmy_server::{init_logging, start_lemmy_server};
use lemmy_utils::{error::LemmyError, settings::SETTINGS};

#[actix_web::main]
pub async fn main() -> Result<(), LemmyError> {
  init_logging(&SETTINGS.opentelemetry_url)?;
  #[cfg(not(feature = "embed-pictrs"))]
  start_lemmy_server().await?;
  #[cfg(feature = "embed-pictrs")]
  {
    pict_rs::init_config::<String, String>(None, None).unwrap();
    let (lemmy, pictrs) = tokio::join!(start_lemmy_server(), pict_rs::run());
    lemmy?;
    pictrs.expect("run pictrs");
  }
  Ok(())
}
