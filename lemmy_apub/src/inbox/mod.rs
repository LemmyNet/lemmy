use crate::{
  check_is_apub_id_valid,
  extensions::signatures::verify_signature,
  fetcher::get_or_fetch_and_upsert_actor,
  ActorType,
};
use activitystreams::{
  activity::ActorAndObjectRefExt,
  base::{AsBase, BaseExt, Extends},
  object::{AsObject, ObjectExt},
  public,
};
use actix_web::HttpRequest;
use anyhow::{anyhow, Context};
use lemmy_db::{activity::Activity, DbPool};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use serde::{export::fmt::Debug, Serialize};
use url::Url;

pub mod community_inbox;
mod receive_for_community;
pub mod shared_inbox;
pub mod user_inbox;

pub(crate) fn get_activity_id<T, Kind>(activity: &T, creator_uri: &Url) -> Result<Url, LemmyError>
where
  T: BaseExt<Kind> + Extends<Kind> + Debug,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  let creator_domain = creator_uri.host_str().context(location_info!())?;
  let activity_id = activity.id(creator_domain)?;
  Ok(activity_id.context(location_info!())?.to_owned())
}

pub(crate) async fn is_activity_already_known(
  pool: &DbPool,
  activity_id: &Url,
) -> Result<bool, LemmyError> {
  let activity_id = activity_id.to_string();
  let existing = blocking(pool, move |conn| {
    Activity::read_from_apub_id(&conn, &activity_id)
  })
  .await?;
  match existing {
    Ok(_) => Ok(true),
    Err(_) => Ok(false),
  }
}

pub(crate) fn get_activity_to_and_cc<T, Kind>(activity: &T) -> Result<Vec<Url>, LemmyError>
where
  T: AsBase<Kind> + AsObject<Kind> + ActorAndObjectRefExt,
{
  let mut to_and_cc = vec![];
  if let Some(to) = activity.to() {
    let to = to.to_owned().unwrap_to_vec();
    let mut to = to
      .iter()
      .map(|t| t.as_xsd_any_uri())
      .flatten()
      .map(|t| t.to_owned())
      .collect();
    to_and_cc.append(&mut to);
  }
  if let Some(cc) = activity.cc() {
    let cc = cc.to_owned().unwrap_to_vec();
    let mut cc = cc
      .iter()
      .map(|c| c.as_xsd_any_uri())
      .flatten()
      .map(|c| c.to_owned())
      .collect();
    to_and_cc.append(&mut cc);
  }
  Ok(to_and_cc)
}

pub(crate) fn is_addressed_to_public<T, Kind>(activity: &T) -> Result<(), LemmyError>
where
  T: AsBase<Kind> + AsObject<Kind> + ActorAndObjectRefExt,
{
  let to_and_cc = get_activity_to_and_cc(activity)?;
  if to_and_cc.contains(&public()) {
    Ok(())
  } else {
    Err(anyhow!("Activity is not addressed to public").into())
  }
}

pub(crate) async fn inbox_verify_http_signature<T, Kind>(
  activity: &T,
  context: &LemmyContext,
  request: HttpRequest,
  request_counter: &mut i32,
) -> Result<Box<dyn ActorType>, LemmyError>
where
  T: AsObject<Kind> + ActorAndObjectRefExt + Extends<Kind> + AsBase<Kind>,
  Kind: Serialize,
  <T as Extends<Kind>>::Error: From<serde_json::Error> + Send + Sync + 'static,
{
  let actor_id = activity
    .actor()?
    .to_owned()
    .single_xsd_any_uri()
    .context(location_info!())?;
  check_is_apub_id_valid(&actor_id)?;
  let actor = get_or_fetch_and_upsert_actor(&actor_id, &context, request_counter).await?;
  verify_signature(&request, actor.as_ref())?;
  Ok(actor)
}
