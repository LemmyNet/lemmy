use crate::{check_is_apub_id_valid, extensions::signatures::sign_and_send, ActorType};
use activitystreams::{
  base::{Extends, ExtendsExt},
  object::AsObject,
};
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
use lemmy_utils::{location_info, settings::Settings, LemmyError};
use log::warn;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, future::Future, pin::Pin};
use url::Url;

pub fn send_activity<T, Kind>(
  activity_sender: &QueueHandle,
  activity: T,
  actor: &dyn ActorType,
  to: Vec<Url>,
) -> Result<(), LemmyError>
where
  T: AsObject<Kind>,
  T: Extends<Kind>,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  if !Settings::get().federation.enabled {
    return Ok(());
  }

  let activity = activity.into_any_base()?;
  let serialised_activity = serde_json::to_string(&activity)?;

  for to_url in &to {
    check_is_apub_id_valid(&to_url)?;
  }

  // TODO: it would make sense to create a separate task for each destination server
  let message = SendActivityTask {
    activity: serialised_activity,
    to,
    actor_id: actor.actor_id()?,
    private_key: actor.private_key().context(location_info!())?,
  };

  activity_sender.queue::<SendActivityTask>(message)?;

  Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SendActivityTask {
  activity: String,
  to: Vec<Url>,
  actor_id: Url,
  private_key: String,
}

impl ActixJob for SendActivityTask {
  type State = MyState;
  type Future = Pin<Box<dyn Future<Output = Result<(), Error>>>>;
  const NAME: &'static str = "SendActivityTask";

  const MAX_RETRIES: MaxRetries = MaxRetries::Count(10);
  const BACKOFF: Backoff = Backoff::Exponential(2);

  fn run(self, state: Self::State) -> Self::Future {
    Box::pin(async move {
      for to_url in &self.to {
        let mut headers = BTreeMap::<String, String>::new();
        headers.insert("Content-Type".into(), "application/json".into());
        let result = sign_and_send(
          &state.client,
          headers,
          to_url,
          self.activity.clone(),
          &self.actor_id,
          self.private_key.to_owned(),
        )
        .await;

        if let Err(e) = result {
          warn!("{}", e);
          return Err(anyhow!(
            "Failed to send activity {} to {}",
            &self.activity,
            to_url
          ));
        }
      }
      Ok(())
    })
  }
}

pub fn create_activity_queue() -> QueueHandle {
  // Start the application server. This guards access to to the jobs store
  let queue_handle = create_server(Storage::new());

  // Configure and start our workers
  WorkerConfig::new(|| MyState {
    client: Client::default(),
  })
  .register::<SendActivityTask>()
  .start(queue_handle.clone());

  queue_handle
}

#[derive(Clone)]
struct MyState {
  pub client: Client,
}
