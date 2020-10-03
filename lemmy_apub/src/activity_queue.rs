use crate::{
  check_is_apub_id_valid,
  community::do_announce,
  extensions::signatures::sign_and_send,
  insert_activity,
  ActorType,
};
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
use itertools::Itertools;
use lemmy_db::{community::Community, user::User_, DbPool};
use lemmy_utils::{location_info, settings::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use log::warn;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, future::Future, pin::Pin};
use url::Url;

pub async fn send_activity_single_dest<T, Kind>(
  activity: T,
  creator: &dyn ActorType,
  to: Url,
  context: &LemmyContext,
) -> Result<(), LemmyError>
where
  T: AsObject<Kind> + Extends<Kind>,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  if check_is_apub_id_valid(&to).is_ok() {
    send_activity_internal(
      context.activity_queue(),
      activity,
      creator,
      vec![to],
      context.pool(),
    )
    .await?;
  }

  Ok(())
}

pub async fn send_to_community_followers<T, Kind>(
  activity: T,
  community: &Community,
  context: &LemmyContext,
  sender_shared_inbox: Option<Url>,
) -> Result<(), LemmyError>
where
  T: AsObject<Kind> + Extends<Kind>,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  // dont send to the local instance, nor to the instance where the activity originally came from,
  // because that would result in a database error (same data inserted twice)
  let community_shared_inbox = community.get_shared_inbox_url()?;
  let to: Vec<Url> = community
    .get_follower_inboxes(context.pool())
    .await?
    .iter()
    .filter(|inbox| Some(inbox) != sender_shared_inbox.as_ref().as_ref())
    .filter(|inbox| inbox != &&community_shared_inbox)
    .filter(|inbox| check_is_apub_id_valid(inbox).is_ok())
    .unique()
    .map(|inbox| inbox.to_owned())
    .collect();

  send_activity_internal(
    context.activity_queue(),
    activity,
    community,
    to,
    context.pool(),
  )
  .await?;

  Ok(())
}

pub async fn send_to_community<T, Kind>(
  creator: &User_,
  community: &Community,
  activity: T,
  context: &LemmyContext,
) -> Result<(), LemmyError>
where
  T: AsObject<Kind> + Extends<Kind>,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  // if this is a local community, we need to do an announce from the community instead
  if community.local {
    do_announce(activity.into_any_base()?, &community, creator, context).await?;
  } else {
    let inbox = community.get_shared_inbox_url()?;
    check_is_apub_id_valid(&inbox)?;
    send_activity_internal(
      context.activity_queue(),
      activity,
      creator,
      vec![inbox],
      context.pool(),
    )
    .await?;
  }

  Ok(())
}

pub async fn send_comment_mentions<T, Kind>(
  creator: &User_,
  mentions: Vec<Url>,
  activity: T,
  context: &LemmyContext,
) -> Result<(), LemmyError>
where
  T: AsObject<Kind> + Extends<Kind>,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  let mentions = mentions
    .iter()
    .filter(|inbox| check_is_apub_id_valid(inbox).is_ok())
    .map(|i| i.to_owned())
    .collect();
  send_activity_internal(
    context.activity_queue(),
    activity,
    creator,
    mentions,
    context.pool(),
  )
  .await?;
  Ok(())
}

/// Asynchronously sends the given `activity` from `actor` to every inbox URL in `to`.
///
/// The caller of this function needs to remove any blocked domains from `to`,
/// using `check_is_apub_id_valid()`.
async fn send_activity_internal<T, Kind>(
  activity_sender: &QueueHandle,
  activity: T,
  actor: &dyn ActorType,
  to: Vec<Url>,
  pool: &DbPool,
) -> Result<(), LemmyError>
where
  T: AsObject<Kind> + Extends<Kind>,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  if !Settings::get().federation.enabled {
    return Ok(());
  }

  for to_url in &to {
    assert!(check_is_apub_id_valid(&to_url).is_ok());
  }

  let activity = activity.into_any_base()?;
  let serialised_activity = serde_json::to_string(&activity)?;
  insert_activity(actor.user_id(), activity.clone(), true, pool).await?;

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
