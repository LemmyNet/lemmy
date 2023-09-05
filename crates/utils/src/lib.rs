#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate smart_default;

pub mod apub;
pub mod cache_header;
pub mod email;
pub mod rate_limit;
pub mod settings;

pub mod claims;
pub mod error;
pub mod request;
pub mod response;
pub mod utils;
pub mod version;

use error::LemmyError;
use futures::Future;
use once_cell::sync::Lazy;
use std::time::Duration;
use tracing::Instrument;

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

/// if true, all federation should happen synchronously. useful for debugging and testing.
/// defaults to true on debug mode, false on releasemode
/// override to true by setting env LEMMY_SYNCHRONOUS_FEDERATION=1
/// override to false by setting env LEMMY_SYNCHRONOUS_FEDERATION=""
pub static SYNCHRONOUS_FEDERATION: Lazy<bool> = Lazy::new(|| {
    std::env::var("LEMMY_SYNCHRONOUS_FEDERATION")
        .map(|s| !s.is_empty())
        .unwrap_or(cfg!(debug_assertions))
});

/// tokio::spawn, but accepts a future that may fail and also
/// * logs errors
/// * attaches the spawned task to the tracing span of the caller for better logging
pub fn spawn_try_task(task: impl Future<Output = Result<(), LemmyError>> + Send + 'static) {
    tokio::spawn(
        async {
            if let Err(e) = task.await {
                tracing::warn!("error in spawn: {e}");
            }
        }
        .in_current_span(), // this makes sure the inner tracing gets the same context as where spawn was called
    );
}
