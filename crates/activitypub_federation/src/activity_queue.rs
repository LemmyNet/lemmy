use crate::{
  signatures::{sign_request, PublicKey},
  Error,
  LocalInstance,
  APUB_JSON_CONTENT_TYPE,
};
use anyhow::anyhow;
use background_jobs::{
  memory_storage::Storage,
  ActixJob,
  Backoff,
  Manager,
  MaxRetries,
  WorkerConfig,
};
use http::{header::HeaderName, HeaderMap, HeaderValue};
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, future::Future, pin::Pin, time::Duration};
use tracing::{info, warn};
use url::Url;

/// Necessary data for sending out an activity
#[derive(Debug)]
pub struct SendActivity {
  /// Id of the sent activity, used for logging
  pub activity_id: Url,
  /// Public key and actor id of the sender
  pub actor_public_key: PublicKey,
  /// Signing key of sender for HTTP signatures
  pub actor_private_key: String,
  /// List of Activitypub inboxes that the activity gets delivered to
  pub inboxes: Vec<Url>,
  /// Activity json
  pub activity: String,
}

impl SendActivity {
  /// Send out the given activity to all inboxes, automatically generating the HTTP signatures. By
  /// default, sending is done on a background thread, and automatically retried on failure with
  /// exponential backoff.
  ///
  /// For debugging or testing, you might want to set [[InstanceSettings.testing_send_sync]].
  pub async fn send(self, instance: &LocalInstance) -> Result<(), Error> {
    let activity_queue = &instance.activity_queue;
    for inbox in self.inboxes {
      let message = SendActivityTask {
        activity_id: self.activity_id.clone(),
        inbox,
        activity: self.activity.clone(),
        public_key: self.actor_public_key.clone(),
        private_key: self.actor_private_key.clone(),
      };
      if instance.settings.testing_send_sync {
        let res = do_send(message, &instance.client, instance.settings.request_timeout).await;
        // Don't fail on error, as we intentionally do some invalid actions in tests, to verify that
        // they are rejected on the receiving side. These errors shouldn't bubble up to make the API
        // call fail. This matches the behaviour in production.
        if let Err(e) = res {
          warn!("{}", e);
        }
      } else {
        activity_queue.queue::<SendActivityTask>(message).await?;
        let stats = activity_queue.get_stats().await?;
        info!(
        "Activity queue stats: pending: {}, running: {}, dead (this hour): {}, complete (this hour): {}",
        stats.pending,
        stats.running,
        stats.dead.this_hour(),
        stats.complete.this_hour()
      );
      }
    }

    Ok(())
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SendActivityTask {
  activity_id: Url,
  inbox: Url,
  activity: String,
  public_key: PublicKey,
  private_key: String,
}

/// Signs the activity with the sending actor's key, and delivers to the given inbox. Also retries
/// if the delivery failed.
impl ActixJob for SendActivityTask {
  type State = MyState;
  type Future = Pin<Box<dyn Future<Output = Result<(), anyhow::Error>>>>;
  const NAME: &'static str = "SendActivityTask";

  /// With these params, retries are made at the following intervals:
  ///          3s
  ///          9s
  ///         27s
  ///      1m 21s
  ///      4m  3s
  ///     12m  9s
  ///     36m 27s
  ///  1h 49m 21s
  ///  5h 28m  3s
  /// 16h 24m  9s
  const MAX_RETRIES: MaxRetries = MaxRetries::Count(10);
  const BACKOFF: Backoff = Backoff::Exponential(3);

  fn run(self, state: Self::State) -> Self::Future {
    Box::pin(async move { do_send(self, &state.client, state.timeout).await })
  }
}

async fn do_send(
  task: SendActivityTask,
  client: &ClientWithMiddleware,
  timeout: Duration,
) -> Result<(), anyhow::Error> {
  info!("Sending {} to {}", task.activity_id, task.inbox);
  let request_builder = client
    .post(&task.inbox.to_string())
    .timeout(timeout)
    .headers(generate_request_headers(&task.inbox));
  let request = sign_request(
    request_builder,
    task.activity.clone(),
    task.public_key.clone(),
    task.private_key.to_owned(),
  )
  .await?;
  let response = client.execute(request).await;

  match response {
    Ok(o) => {
      if o.status().is_success() {
        Ok(())
      } else {
        let status = o.status();
        let text = o.text().await.map_err(Error::conv)?;

        Err(anyhow!(
          "Send {} to {} failed with status {}: {}",
          task.activity_id,
          task.inbox,
          status,
          text,
        ))
      }
    }
    Err(e) => Err(anyhow!(
      "Failed to send activity {} to {}: {}",
      &task.activity_id,
      task.inbox,
      e
    )),
  }
}

fn generate_request_headers(inbox_url: &Url) -> HeaderMap {
  let mut host = inbox_url.domain().expect("read inbox domain").to_string();
  if let Some(port) = inbox_url.port() {
    host = format!("{}:{}", host, port);
  }

  let mut headers = HeaderMap::new();
  headers.insert(
    HeaderName::from_static("content-type"),
    HeaderValue::from_static(APUB_JSON_CONTENT_TYPE),
  );
  headers.insert(
    HeaderName::from_static("host"),
    HeaderValue::from_str(&host).expect("Hostname is valid"),
  );
  headers
}

pub(crate) fn create_activity_queue(
  client: ClientWithMiddleware,
  worker_count: u64,
  timeout: Duration,
) -> Manager {
  // Configure and start our workers
  WorkerConfig::new_managed(Storage::new(), move |_| MyState {
    client: client.clone(),
    timeout,
  })
  .register::<SendActivityTask>()
  .set_worker_count("default", worker_count)
  .start()
}

#[derive(Clone)]
struct MyState {
  client: ClientWithMiddleware,
  timeout: Duration,
}
