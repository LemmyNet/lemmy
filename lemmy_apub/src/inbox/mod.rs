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
use lemmy_db_queries::{source::activity::Activity_, ApubObject, DbPool};
use lemmy_db_schema::source::{activity::Activity, community::Community, user::User_};
use lemmy_structs::blocking;
use lemmy_utils::{location_info, settings::Settings, LemmyError};
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

pub(crate) fn get_activity_to_and_cc<T, Kind>(activity: &T) -> Vec<Url>
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
  to_and_cc
}

pub(crate) fn is_addressed_to_public<T, Kind>(activity: &T) -> Result<(), LemmyError>
where
  T: AsBase<Kind> + AsObject<Kind> + ActorAndObjectRefExt,
{
  let to_and_cc = get_activity_to_and_cc(activity);
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

/// Returns true if `to_and_cc` contains at least one local user.
pub(crate) async fn is_addressed_to_local_user(
  to_and_cc: &[Url],
  pool: &DbPool,
) -> Result<bool, LemmyError> {
  for url in to_and_cc {
    let url = url.to_string();
    let user = blocking(&pool, move |conn| User_::read_from_apub_id(&conn, &url)).await?;
    if let Ok(u) = user {
      if u.local {
        return Ok(true);
      }
    }
  }
  Ok(false)
}

/// If `to_and_cc` contains the followers collection of a remote community, returns this community
/// (like `https://example.com/c/main/followers`)
pub(crate) async fn is_addressed_to_community_followers(
  to_and_cc: &[Url],
  pool: &DbPool,
) -> Result<Option<Community>, LemmyError> {
  for url in to_and_cc {
    let url = url.to_string();
    // TODO: extremely hacky, we should just store the followers url for each community in the db
    if url.ends_with("/followers") {
      let community_url = url.replace("/followers", "");
      let community = blocking(&pool, move |conn| {
        Community::read_from_apub_id(&conn, &community_url)
      })
      .await??;
      if !community.local {
        return Ok(Some(community));
      }
    }
  }
  Ok(None)
}

pub(in crate::inbox) fn assert_activity_not_local<T, Kind>(activity: &T) -> Result<(), LemmyError>
where
  T: BaseExt<Kind> + Debug,
{
  let id = activity.id_unchecked().context(location_info!())?;
  let activity_domain = id.domain().context(location_info!())?;

  if activity_domain == Settings::get().hostname {
    return Err(
      anyhow!(
        "Error: received activity which was sent by local instance: {:?}",
        activity
      )
      .into(),
    );
  }
  Ok(())
}
