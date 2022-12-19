use lemmy_server::{init_logging, start_lemmy_server};
use lemmy_utils::{error::LemmyError, settings::SETTINGS};

#[actix_web::main]
pub async fn main() -> Result<(), LemmyError> {
  init_logging(&SETTINGS.opentelemetry_url)?;
  start_lemmy_server().await
}
