#[cfg(feature = "apub")]
pub mod apub;
#[cfg(feature = "cache-header")]
pub mod cache_header;
#[cfg(feature = "email")]
pub mod email;
#[cfg(feature = "error-type")]
pub mod error;
#[cfg(feature = "rate-limit")]
pub mod rate_limit;
#[cfg(feature = "request")]
pub mod request;
#[cfg(feature = "response")]
pub mod response;
#[cfg(feature = "settings")]
pub mod settings;
#[cfg(feature = "misc")]
pub mod utils;
pub mod version;

use std::time::Duration;

pub type ConnectionId = usize;

pub const REQWEST_TIMEOUT: Duration = Duration::from_secs(10);

#[macro_export]
macro_rules! location_info {
  () => {
    format!(
      "None value at {}:{}, column {}",
      file!(),
      line!(),
      column!()
    )
  };
}

#[cfg(feature = "misc")]
/// tokio::spawn, but accepts a future that may fail and also
/// * logs errors
/// * attaches the spawned task to the tracing span of the caller for better logging
pub fn spawn_try_task(
  task: impl futures::Future<Output = Result<(), error::LemmyError>> + Send + 'static,
) {
  use tracing::Instrument;
  tokio::spawn(
    async {
      if let Err(e) = task.await {
        tracing::warn!("error in spawn: {e}");
      }
    }
    .in_current_span(), // this makes sure the inner tracing gets the same context as where spawn was called
  );
}
