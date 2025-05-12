use cfg_if::cfg_if;

cfg_if! {
  if #[cfg(feature = "full")] {
    pub mod cache_header;
    pub mod rate_limit;
    pub mod request;
    pub mod response;
    pub mod settings;
    pub mod utils;
  }
}

pub mod error;
use std::time::Duration;

pub type ConnectionId = usize;

/// git_version marks this crate as dirty and causes a rebuild if any file in the repo is changed.
/// This slows down development a lot, so we only use git_version for release builds.
#[cfg(not(debug_assertions))]
pub const VERSION: &str = git_version::git_version!(
  args = ["--tags", "--dirty=-modified"],
  fallback = env!("CARGO_PKG_VERSION")
);
#[cfg(debug_assertions)]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const REQWEST_TIMEOUT: Duration = Duration::from_secs(10);

#[cfg(debug_assertions)]
pub const CACHE_DURATION_FEDERATION: Duration = Duration::from_millis(500);
#[cfg(not(debug_assertions))]
pub const CACHE_DURATION_FEDERATION: Duration = Duration::from_secs(60);

#[cfg(debug_assertions)]
pub const CACHE_DURATION_API: Duration = Duration::from_secs(0);
#[cfg(not(debug_assertions))]
pub const CACHE_DURATION_API: Duration = Duration::from_secs(1);

#[cfg(debug_assertions)]
pub const CACHE_DURATION_LARGEST_COMMUNITY: Duration = Duration::from_secs(0);
#[cfg(not(debug_assertions))]
pub const CACHE_DURATION_LARGEST_COMMUNITY: Duration = Duration::from_days(1);

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

cfg_if! {
  if #[cfg(feature = "full")] {
use moka::future::Cache;use std::fmt::Debug;use std::hash::Hash;
use serde_json::Value;use std::{sync::LazyLock};

/// Only include a basic context to save space and bandwidth. The main context is hosted statically
/// on join-lemmy.org. Include activitystreams explicitly for better compat, but this could
/// theoretically also be moved.
pub static FEDERATION_CONTEXT: LazyLock<Value> = LazyLock::new(|| {
  Value::Array(vec![
    Value::String("https://join-lemmy.org/context.json".to_string()),
    Value::String("https://www.w3.org/ns/activitystreams".to_string()),
  ])
});

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

pub fn build_cache<K, V>() -> Cache<K, V>
where
  K: Debug + Eq + Hash + Send + Sync + 'static,
  V: Debug + Clone + Send + Sync + 'static,
{
  Cache::<K, V>::builder()
    .max_capacity(1)
    .time_to_live(CACHE_DURATION_API)
    .build()
}

#[cfg(feature = "full")]
pub type CacheLock<T> = std::sync::LazyLock<Cache<(), T>>;

  }
}
