use crate::activity_queue::create_activity_queue;
use background_jobs::Manager;
use derive_builder::Builder;
use reqwest_middleware::ClientWithMiddleware;
use std::time::Duration;
use url::Url;

pub mod activity_queue;
pub mod context;
pub mod data;
pub mod deser;
pub mod inbox;
pub mod object_id;
pub mod signatures;
pub mod traits;
pub mod utils;
pub mod values;
pub mod verify;

/// Mime type for Activitypub, used for `Accept` and `Content-Type` HTTP headers
pub static APUB_JSON_CONTENT_TYPE: &str = "application/activity+json";

/// Represents a single, federated instance (for example lemmy.ml). There should only be one of
/// this in your application (except for testing).
pub struct LocalInstance {
  hostname: String,
  client: ClientWithMiddleware,
  activity_queue: Manager,
  settings: InstanceSettings,
}

// Use InstanceSettingsBuilder to initialize this
#[derive(Builder)]
pub struct InstanceSettings {
  /// Maximum number of outgoing HTTP requests per incoming activity
  #[builder(default = "20")]
  http_fetch_retry_limit: i32,
  /// Number of worker threads for sending outgoing activities
  #[builder(default = "64")]
  worker_count: u64,
  /// Send outgoing activities synchronously, not in background thread. Helps to make tests
  /// more consistent, but not recommended for production.
  #[builder(default = "false")]
  testing_send_sync: bool,
  /// Timeout for all HTTP requests. HTTP signatures are valid for 10s, so it makes sense to
  /// use the same as timeout when sending
  #[builder(default = "Duration::from_secs(10)")]
  request_timeout: Duration,
  /// Function used to verify that urls are valid, used when receiving activities or fetching remote
  /// objects. Use this to implement functionality like federation blocklists. In case verification
  /// fails, it should return an error message.
  #[builder(default = "|_| { Ok(()) }")]
  verify_url_function: fn(&Url) -> Result<(), &'static str>,
}

impl LocalInstance {
  pub fn new(domain: String, client: ClientWithMiddleware, settings: InstanceSettings) -> Self {
    let activity_queue = create_activity_queue(
      client.clone(),
      settings.worker_count,
      settings.request_timeout,
    );
    LocalInstance {
      hostname: domain,
      client,
      activity_queue,
      settings,
    }
  }
  /// Returns true if the url refers to this instance. Handles hostnames like `localhost:8540` for
  /// local debugging.
  fn is_local_url(&self, url: &Url) -> bool {
    let mut domain = url.domain().expect("id has domain").to_string();
    if let Some(port) = url.port() {
      domain = format!("{}:{}", domain, port);
    }
    domain == self.hostname
  }

  /// Returns the local hostname
  pub fn hostname(&self) -> &str {
    &self.hostname
  }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("Object was not found in database")]
  NotFound,
  #[error("Request limit was reached during fetch")]
  RequestLimit,
  #[error("Object to be fetched was deleted")]
  ObjectDeleted,
  #[error("{0}")]
  UrlVerificationError(&'static str),
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
