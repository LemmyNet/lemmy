use crate::{
  signatures::sign_request,
  traits::ActorType,
  Error,
  Error::PrivateKeyError,
  APUB_JSON_CONTENT_TYPE,
};
use anyhow::anyhow;
use background_jobs::{
  memory_storage::Storage,
  ActixJob,
  Backoff,
  Manager,
  MaxRetries,
  QueueHandle,
  WorkerConfig,
};
use http::{header::HeaderName, HeaderMap, HeaderValue};
use lemmy_utils::REQWEST_TIMEOUT;
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
use std::{env, fmt::Debug, future::Future, pin::Pin};
use tracing::{info, warn};
use url::Url;

pub async fn send_activity(
  activity_id: &Url,
  actor: &dyn ActorType,
  inboxes: Vec<&Url>,
  activity: String,
  client: &ClientWithMiddleware,
  activity_queue: &QueueHandle,
) -> Result<(), Error> {
  for i in inboxes {
    let message = SendActivityTask {
      activity_id: activity_id.clone(),
      inbox: i.to_owned(),
      actor_id: actor.actor_id(),
      activity: activity.clone(),
      private_key: actor.private_key().ok_or(PrivateKeyError)?,
    };
    if env::var("APUB_TESTING_SEND_SYNC").is_ok() {
      let res = do_send(message, client).await;
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

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SendActivityTask {
  activity_id: Url,
  inbox: Url,
  actor_id: Url,
  activity: String,
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
    Box::pin(async move { do_send(self, &state.client).await })
  }
}

async fn do_send(
  task: SendActivityTask,
  client: &ClientWithMiddleware,
) -> Result<(), anyhow::Error> {
  info!("Sending {} to {}", task.activity_id, task.inbox);
  let request_builder = client
    .post(&task.inbox.to_string())
    // signature is only valid for 10 seconds, so no reason to wait any longer
    // TODO: would be good if we could get expiration directly from
    //       http_signature_normalization::Config, but its private
    .timeout(REQWEST_TIMEOUT)
    .headers(generate_request_headers(&task.inbox));
  let request = sign_request(
    request_builder,
    task.activity.clone(),
    &task.actor_id,
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
    HeaderName::from_static("Content-Type"),
    HeaderValue::from_static(APUB_JSON_CONTENT_TYPE),
  );
  headers.insert(
    HeaderName::from_static("Host"),
    HeaderValue::from_str(&host).expect("Hostname is valid"),
  );
  headers
}

pub fn create_activity_queue(client: ClientWithMiddleware, worker_count: u64) -> Manager {
  // Configure and start our workers
  WorkerConfig::new_managed(Storage::new(), move |_| MyState {
    client: client.clone(),
  })
  .register::<SendActivityTask>()
  .set_worker_count("default", worker_count)
  .start()
}

#[derive(Clone)]
struct MyState {
  pub client: ClientWithMiddleware,
}
