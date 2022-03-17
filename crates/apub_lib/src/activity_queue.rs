use crate::{signatures::sign_and_send, traits::ActorType};
use anyhow::{anyhow, Context, Error};
use background_jobs::{
  memory_storage::Storage,
  ActixJob,
  Backoff,
  Manager,
  MaxRetries,
  QueueHandle,
  WorkerConfig,
};
use lemmy_utils::{location_info, LemmyError};
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
) -> Result<(), LemmyError> {
  for i in inboxes {
    let message = SendActivityTask {
      activity_id: activity_id.clone(),
      inbox: i.to_owned(),
      actor_id: actor.actor_id(),
      activity: activity.clone(),
      private_key: actor.private_key().context(location_info!())?,
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
  type Future = Pin<Box<dyn Future<Output = Result<(), Error>>>>;
  const NAME: &'static str = "SendActivityTask";

  const MAX_RETRIES: MaxRetries = MaxRetries::Count(10);
  const BACKOFF: Backoff = Backoff::Exponential(2);

  fn run(self, state: Self::State) -> Self::Future {
    Box::pin(async move { do_send(self, &state.client).await })
  }
}

async fn do_send(task: SendActivityTask, client: &ClientWithMiddleware) -> Result<(), Error> {
  info!("Sending {} to {}", task.activity_id, task.inbox);
  let result = sign_and_send(
    client,
    &task.inbox,
    task.activity.clone(),
    &task.actor_id,
    task.private_key.to_owned(),
  )
  .await;

  let r: Result<(), Error> = match result {
    Ok(o) => {
      if !o.status().is_success() {
        let status = o.status();
        let text = o.text().await?;

        Err(anyhow!(
          "Send {} to {} failed with status {}: {}",
          task.activity_id,
          task.inbox,
          status,
          text,
        ))
      } else {
        Ok(())
      }
    }
    Err(e) => Err(anyhow!(
      "Failed to send activity {} to {}: {}",
      &task.activity_id,
      task.inbox,
      e
    )),
  };
  r
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
