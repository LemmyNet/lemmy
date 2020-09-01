use crate::{
  apub::{activity_queue::send_activity, community::do_announce, insert_activity},
  LemmyContext,
};
use activitystreams::{
  base::{Extends, ExtendsExt},
  object::AsObject,
};
use lemmy_db::{community::Community, user::User_};
use lemmy_utils::{get_apub_protocol_string, settings::Settings, LemmyError};
use serde::{export::fmt::Debug, Serialize};
use url::{ParseError, Url};
use uuid::Uuid;

pub async fn send_activity_to_community<T, Kind>(
  creator: &User_,
  community: &Community,
  to: Vec<Url>,
  activity: T,
  context: &LemmyContext,
) -> Result<(), LemmyError>
where
  T: AsObject<Kind> + Extends<Kind> + Serialize + Debug + Send + Clone + 'static,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  // TODO: looks like call this sometimes with activity, and sometimes with any_base
  insert_activity(creator.id, activity.clone(), true, context.pool()).await?;

  // if this is a local community, we need to do an announce from the community instead
  if community.local {
    do_announce(activity.into_any_base()?, &community, creator, context).await?;
  } else {
    send_activity(context.activity_queue(), activity, creator, to)?;
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
