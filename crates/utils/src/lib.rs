use cfg_if::cfg_if;

cfg_if! {
  if #[cfg(feature = "default")] {
    pub mod apub;
    pub mod cache_header;
    pub mod email;
    pub mod error;
    pub mod rate_limit;
    pub mod request;
    pub mod response;
    pub mod settings;
    pub mod utils;
    pub mod version;
    pub use error::LemmyErrorType;
  } else {
    mod error;
  }
}

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

#[cfg(feature = "default")]
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
