use crate::{
  apub::{
    check_is_apub_id_valid,
    community::do_announce,
    extensions::signatures::sign,
    insert_activity,
    ActorType,
  },
  request::retry_custom,
  DbPool,
  LemmyError,
};
use activitystreams::base::AnyBase;
use actix_web::client::Client;
use lemmy_db::{community::Community, user::User_};
use lemmy_utils::{get_apub_protocol_string, settings::Settings};
use log::debug;
use url::{ParseError, Url};
use uuid::Uuid;

pub async fn send_activity_to_community(
  creator: &User_,
  community: &Community,
  to: Vec<String>,
  activity: AnyBase,
  client: &Client,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  insert_activity(creator.id, activity.clone(), true, pool).await?;

  // if this is a local community, we need to do an announce from the community instead
  if community.local {
    do_announce(activity, &community, creator, client, pool).await?;
  } else {
    send_activity(client, &activity, creator, to).await?;
  }

  Ok(())
}

/// Send an activity to a list of recipients, using the correct headers etc.
pub async fn send_activity(
  client: &Client,
  activity: &AnyBase,
  actor: &dyn ActorType,
  to: Vec<String>,
) -> Result<(), LemmyError> {
  let activity = serde_json::to_string(&activity)?;
  debug!("Sending activitypub activity {} to {:?}", activity, to);

  for t in to {
    let to_url = Url::parse(&t)?;
    check_is_apub_id_valid(&to_url)?;

    let res = retry_custom(|| async {
      let request = client.post(&t).header("Content-Type", "application/json");

      match sign(request, actor, activity.clone()).await {
        Ok(signed) => Ok(signed.send().await),
        Err(e) => Err(e),
      }
    })
    .await?;

    debug!("Result for activity send: {:?}", res);
  }

  Ok(())
}

pub(in crate::apub) fn generate_activity_id<T>(kind: T) -> Result<Url, ParseError>
where
  T: ToString,
{
  let id = format!(
    "{}://{}/activities/{}/{}",
    get_apub_protocol_string(),
    Settings::get().hostname,
    kind.to_string().to_lowercase(),
    Uuid::new_v4()
  );
  Url::parse(&id)
}
