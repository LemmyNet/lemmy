use crate::{signatures::sign_and_send, traits::ActorType};
use anyhow::{anyhow, Context, Error};
use background_jobs::{
  create_server,
  memory_storage::Storage,
  ActixJob,
  Backoff,
  MaxRetries,
  QueueHandle,
  WorkerConfig,
};
use lemmy_utils::{location_info, LemmyError};
use log::warn;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, fmt::Debug, future::Future, pin::Pin};
use url::Url;

pub async fn send_activity(
  activity: String,
  actor: &dyn ActorType,
  inboxes: Vec<&Url>,
  client: &Client,
  activity_queue: &QueueHandle,
) -> Result<(), LemmyError> {
  for i in inboxes {
    let message = SendActivityTask {
      activity: activity.clone(),
      inbox: i.to_owned(),
      actor_id: actor.actor_id(),
      private_key: actor.private_key().context(location_info!())?,
    };
    if env::var("APUB_TESTING_SEND_SYNC").is_ok() {
      do_send(message, client).await?;
    } else {
      activity_queue.queue::<SendActivityTask>(message)?;
    }
  }

  Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SendActivityTask {
  activity: String,
  inbox: Url,
  actor_id: Url,
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

async fn do_send(task: SendActivityTask, client: &Client) -> Result<(), Error> {
  let result = sign_and_send(
    client,
    &task.inbox,
    task.activity.clone(),
    &task.actor_id,
    task.private_key.to_owned(),
  )
  .await;

  if let Err(e) = result {
    warn!("{}", e);
    return Err(anyhow!(
      "Failed to send activity {} to {}",
      &task.activity,
      task.inbox
    ));
  }
  Ok(())
}

pub fn create_activity_queue() -> QueueHandle {
  // Start the application server. This guards access to to the jobs store
  let queue_handle = create_server(Storage::new());
  let arbiter = actix_web::rt::Arbiter::new();

  // Configure and start our workers
  WorkerConfig::new(|| MyState {
    client: Client::default(),
  })
  .register::<SendActivityTask>()
  .start_in_arbiter(&arbiter, queue_handle.clone());

  queue_handle
}

#[derive(Clone)]
struct MyState {
  pub client: Client,
}
