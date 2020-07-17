use crate::{
  apub::{
    community::do_announce, extensions::signatures::sign, insert_activity, is_apub_id_valid,
    ActorType,
  },
  request::retry_custom,
  DbPool, LemmyError,
};
use activitystreams_new::base::AnyBase;
use actix_web::client::Client;
use lemmy_db::{community::Community, user::User_};
use log::debug;
use url::Url;

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
    if !is_apub_id_valid(&to_url) {
      debug!("Not sending activity to {} (invalid or blocklisted)", t);
      continue;
    }

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
