use cfg_if::cfg_if;

cfg_if! {
  if #[cfg(feature = "full")] {
    pub mod cache_header;
    pub mod email;
    pub mod rate_limit;
    pub mod request;
    pub mod response;
    pub mod settings;
    pub mod utils;
  }
}

pub mod error;
pub use error::LemmyErrorType;
use std::time::Duration;

pub type ConnectionId = usize;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const REQWEST_TIMEOUT: Duration = Duration::from_secs(10);

#[cfg(debug_assertions)]
pub const CACHE_DURATION_FEDERATION: Duration = Duration::from_millis(500);
#[cfg(not(debug_assertions))]
pub const CACHE_DURATION_FEDERATION: Duration = Duration::from_secs(60);

pub const CACHE_DURATION_API: Duration = Duration::from_secs(1);

pub const MAX_COMMENT_DEPTH_LIMIT: usize = 50;

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

#[cfg(feature = "full")]
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
    .in_current_span(), /* this makes sure the inner tracing gets the same context as where
                         * spawn was called */
  );
}
