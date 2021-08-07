use crate::{
  activities::community::announce::{AnnouncableActivities, AnnounceActivity},
  check_is_apub_id_valid,
  extensions::signatures::sign_and_send,
  insert_activity,
  ActorType,
  CommunityType,
  APUB_JSON_CONTENT_TYPE,
};
use activitystreams::{
  base::{BaseExt, Extends, ExtendsExt},
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
use lemmy_db_schema::source::{community::Community, person::Person};
use lemmy_utils::{location_info, settings::structs::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use log::{debug, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, env, fmt::Debug, future::Future, pin::Pin};
use url::Url;

/// From a local community, send activity to all remote followers.
///
/// * `activity` the apub activity to send
/// * `community` the sending community
/// * `extra_inbox` actor inbox which should receive the activity, in addition to followers
pub(crate) async fn send_to_community_followers<T, Kind>(
  activity: T,
  community: &Community,
  extra_inbox: Option<Url>,
  context: &LemmyContext,
) -> Result<(), LemmyError>
where
  T: AsObject<Kind> + Extends<Kind> + Debug + BaseExt<Kind>,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  let extra_inbox: Vec<Url> = extra_inbox.into_iter().collect();
  let follower_inboxes: Vec<Url> = vec![
    community.get_follower_inboxes(context.pool()).await?,
    extra_inbox,
  ]
  .iter()
  .flatten()
  .unique()
  .filter(|inbox| inbox.host_str() != Some(&Settings::get().hostname))
  .filter(|inbox| check_is_apub_id_valid(inbox, false).is_ok())
  .map(|inbox| inbox.to_owned())
  .collect();
  debug!(
    "Sending activity {:?} to followers of {}",
    &activity.id_unchecked().map(ToString::to_string),
    &community.actor_id
  );

  send_activity_internal(context, activity, community, follower_inboxes, true, false).await?;

  Ok(())
}

/// Sends an activity from a local person to a remote community.
///
/// * `activity` the activity to send
/// * `creator` the creator of the activity
/// * `community` the destination community
/// * `object_actor` if the object of the activity is an actor, it should be passed here so it can
///                  be sent directly to the actor
///
pub(crate) async fn send_to_community<T, Kind>(
  activity: T,
  creator: &Person,
  community: &Community,
  object_actor: Option<Url>,
  context: &LemmyContext,
) -> Result<(), LemmyError>
where
  T: AsObject<Kind> + Extends<Kind> + Debug + BaseExt<Kind>,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  // if this is a local community, we need to do an announce from the community instead
  if community.local {
    community
      .send_announce(activity.into_any_base()?, object_actor, context)
      .await?;
  } else {
    let inbox = community.get_shared_inbox_or_inbox_url();
    check_is_apub_id_valid(&inbox, false)?;
    debug!(
      "Sending activity {:?} to community {}",
      &activity.id_unchecked().map(ToString::to_string),
      &community.actor_id
    );
    // dont send to object_actor here, as that is responsibility of the community itself
    send_activity_internal(context, activity, creator, vec![inbox], true, false).await?;
  }

  Ok(())
}

pub(crate) async fn send_to_community_new(
  activity: AnnouncableActivities,
  activity_id: &Url,
  actor: &dyn ActorType,
  community: &Community,
  additional_inboxes: Vec<Url>,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  // if this is a local community, we need to do an announce from the community instead
  if community.local {
    insert_activity(activity_id, activity.clone(), true, false, context.pool()).await?;
    AnnounceActivity::send(activity, community, additional_inboxes, context).await?;
  } else {
    let mut inboxes = additional_inboxes;
    inboxes.push(community.get_shared_inbox_or_inbox_url());
    send_activity_new(context, &activity, activity_id, actor, inboxes, false).await?;
  }

  Ok(())
}

pub(crate) async fn send_activity_new<T>(
  context: &LemmyContext,
  activity: &T,
  activity_id: &Url,
  actor: &dyn ActorType,
  inboxes: Vec<Url>,
  sensitive: bool,
) -> Result<(), LemmyError>
where
  T: Serialize,
{
  if !Settings::get().federation.enabled || inboxes.is_empty() {
    return Ok(());
  }

  info!("Sending activity {}", activity_id.to_string());

  // Don't send anything to ourselves
  // TODO: this should be a debug assert
  let hostname = Settings::get().get_hostname_without_port()?;
  let inboxes: Vec<&Url> = inboxes
    .iter()
    .filter(|i| i.domain().expect("valid inbox url") != hostname)
    .collect();

  let serialised_activity = serde_json::to_string(&activity)?;

  insert_activity(
    activity_id,
    serialised_activity.clone(),
    true,
    sensitive,
    context.pool(),
  )
  .await?;

  for i in inboxes {
    let message = SendActivityTask {
      activity: serialised_activity.to_owned(),
      inbox: i.to_owned(),
      actor_id: actor.actor_id(),
      private_key: actor.private_key().context(location_info!())?,
    };
    if env::var("LEMMY_TEST_SEND_SYNC").is_ok() {
      do_send(message, &Client::default()).await?;
    } else {
      context.activity_queue.queue::<SendActivityTask>(message)?;
    }
  }

  Ok(())
}

/// Create new `SendActivityTasks`, which will deliver the given activity to inboxes, as well as
/// handling signing and retrying failed deliveres.
///
/// The caller of this function needs to remove any blocked domains from `to`,
/// using `check_is_apub_id_valid()`.
async fn send_activity_internal<T, Kind>(
  context: &LemmyContext,
  activity: T,
  actor: &dyn ActorType,
  inboxes: Vec<Url>,
  insert_into_db: bool,
  sensitive: bool,
) -> Result<(), LemmyError>
where
  T: AsObject<Kind> + Extends<Kind> + Debug,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  if !Settings::get().federation.enabled || inboxes.is_empty() {
    return Ok(());
  }

  // Don't send anything to ourselves
  let hostname = Settings::get().get_hostname_without_port()?;
  let inboxes: Vec<&Url> = inboxes
    .iter()
    .filter(|i| i.domain().expect("valid inbox url") != hostname)
    .collect();

  let activity = activity.into_any_base()?;
  let serialised_activity = serde_json::to_string(&activity)?;

  // This is necessary because send_comment and send_comment_mentions
  // might send the same ap_id
  if insert_into_db {
    let id = activity.id().context(location_info!())?;
    insert_activity(id, activity.clone(), true, sensitive, context.pool()).await?;
  }

  for i in inboxes {
    let message = SendActivityTask {
      activity: serialised_activity.to_owned(),
      inbox: i.to_owned(),
      actor_id: actor.actor_id(),
      private_key: actor.private_key().context(location_info!())?,
    };
    if env::var("LEMMY_TEST_SEND_SYNC").is_ok() {
      do_send(message, &Client::default()).await?;
    } else {
      context.activity_queue.queue::<SendActivityTask>(message)?;
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
  let mut headers = BTreeMap::<String, String>::new();
  headers.insert("Content-Type".into(), APUB_JSON_CONTENT_TYPE.to_string());
  let result = sign_and_send(
    client,
    headers,
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
