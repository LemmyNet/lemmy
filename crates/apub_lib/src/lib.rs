use crate::activity_queue::create_activity_queue;
use background_jobs::Manager;
use reqwest_middleware::ClientWithMiddleware;
use std::time::Duration;

pub mod activity_queue;
pub mod data;
pub mod object_id;
pub mod signatures;
pub mod traits;
pub mod utils;
pub mod values;
pub mod verify;

pub static APUB_JSON_CONTENT_TYPE: &str = "application/activity+json";
/// HTTP signatures are valid for 10s, so it makes sense to use the same as timeout when sending
pub static DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct LocalInstance {
  pub domain: String,
  client: ClientWithMiddleware,
  activity_queue: Manager,
  settings: InstanceSettings,
}

pub struct InstanceSettings {
  /// Maximum number of outgoing HTTP requests per incoming activity
  http_fetch_retry_limit: i32,
  /// Number of worker threads for sending outgoing activities
  worker_count: u64,
  /// Send outgoing activities synchronously, not in background thread. Helps to make tests
  /// more consistent, but not recommended for production.
  testing_send_sync: bool,
  /// Timeout for all HTTP requests
  request_timeout: Duration,
}

impl LocalInstance {
  pub fn new(domain: String, client: ClientWithMiddleware, settings: InstanceSettings) -> Self {
    let activity_queue = create_activity_queue(
      client.clone(),
      settings.worker_count,
      settings.request_timeout,
    );
    LocalInstance {
      domain,
      client,
      activity_queue,
      settings,
    }
  }
}

impl InstanceSettings {
  pub fn new(
    http_fetch_retry_limit: i32,
    worker_count: u64,
    testing_send_sync: bool,
    request_timeout: Duration,
  ) -> Self {
    InstanceSettings {
      http_fetch_retry_limit,
      worker_count,
      testing_send_sync,
      request_timeout,
    }
  }
}
impl Default for InstanceSettings {
  fn default() -> Self {
    InstanceSettings {
      http_fetch_retry_limit: 20,
      worker_count: 64,
      testing_send_sync: false,
      request_timeout: DEFAULT_TIMEOUT,
    }
  }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Domain did not pass verification")]
  DomainError,
  #[error("Object was not found in database")]
  NotFound,
  #[error("Request limit was reached during fetch")]
  RequestLimit,
  #[error("Object to be fetched was deleted")]
  ObjectDeleted,
  #[error(transparent)]
  Other(#[from] anyhow::Error),
}

impl Error {
  pub fn conv<T>(error: T) -> Self
  where
    T: Into<anyhow::Error>,
  {
    Error::Other(error.into())
  }
}
