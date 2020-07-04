use crate::{
  apub::{extensions::signatures::sign, is_apub_id_valid, ActorType},
  db::{activity::insert_activity, community::Community, user::User_},
  request::retry_custom,
  DbPool,
  LemmyError,
};
use activitystreams::{context, object::properties::ObjectProperties, public, Activity, Base};
use actix_web::client::Client;
use log::debug;
use serde::Serialize;
use std::fmt::Debug;
use url::Url;

pub fn populate_object_props(
  props: &mut ObjectProperties,
  addressed_ccs: Vec<String>,
  object_id: &str,
) -> Result<(), LemmyError> {
  props
    .set_context_xsd_any_uri(context())?
    // TODO: the activity needs a seperate id from the object
    .set_id(object_id)?
    // TODO: should to/cc go on the Create, or on the Post? or on both?
    // TODO: handle privacy on the receiving side (at least ignore anything thats not public)
    .set_to_xsd_any_uri(public())?
    .set_many_cc_xsd_any_uris(addressed_ccs)?;
  Ok(())
}

pub async fn send_activity_to_community<A>(
  creator: &User_,
  community: &Community,
  to: Vec<String>,
  activity: A,
  client: &Client,
  pool: &DbPool,
) -> Result<(), LemmyError>
where
  A: Activity + Base + Serialize + Debug + Clone + Send + 'static,
{
  insert_activity(creator.id, activity.clone(), true, pool).await?;

  // if this is a local community, we need to do an announce from the community instead
  if community.local {
    Community::do_announce(activity, &community, creator, client, pool).await?;
  } else {
    send_activity(client, &activity, creator, to).await?;
  }

  Ok(())
}

/// Send an activity to a list of recipients, using the correct headers etc.
pub async fn send_activity<A>(
  client: &Client,
  activity: &A,
  actor: &dyn ActorType,
  to: Vec<String>,
) -> Result<(), LemmyError>
where
  A: Serialize,
{
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
