use activitystreams::{
  activity::ActorAndObjectRefExt,
  base::{AsBase, Extends},
  object::AsObject,
};
use actix_web::HttpRequest;
use anyhow::Context;
use lemmy_api_common::blocking;
use lemmy_apub::{
  check_is_apub_id_valid,
  extensions::signatures::verify_signature,
  fetcher::get_or_fetch_and_upsert_actor,
  ActorType,
};
use lemmy_db_queries::{source::activity::Activity_, DbPool};
use lemmy_db_schema::source::activity::Activity;
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use serde::Serialize;
use url::Url;

pub mod community_inbox;
pub mod new_inbox_routing;
pub mod person_inbox;
pub mod shared_inbox;

pub(crate) async fn is_activity_already_known(
  pool: &DbPool,
  activity_id: &Url,
) -> Result<bool, LemmyError> {
  let activity_id = activity_id.to_owned().into();
  let existing = blocking(pool, move |conn| {
    Activity::read_from_apub_id(&conn, &activity_id)
  })
  .await?;
  match existing {
    Ok(_) => Ok(true),
    Err(_) => Ok(false),
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
  check_is_apub_id_valid(&actor_id, false)?;
  let actor = get_or_fetch_and_upsert_actor(&actor_id, &context, request_counter).await?;
  verify_signature(&request, actor.as_ref())?;
  Ok(actor)
}
