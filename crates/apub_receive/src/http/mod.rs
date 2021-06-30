use actix_web::{body::Body, web, HttpRequest, HttpResponse};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{activities::LemmyActivity, http::inbox_enums::SharedInboxActivities};
use anyhow::{anyhow, Context};
use lemmy_api_common::blocking;
use lemmy_apub::{
  check_is_apub_id_valid,
  extensions::signatures::verify_signature,
  fetcher::{get_or_fetch_and_upsert_actor, Actor},
  insert_activity,
  APUB_JSON_CONTENT_TYPE,
};
use lemmy_apub_lib::ActivityHandler;
use lemmy_db_queries::{source::activity::Activity_, DbPool};
use lemmy_db_schema::source::activity::Activity;
use lemmy_utils::{location_info, settings::structs::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use std::fmt::Debug;

pub mod comment;
pub mod community;
pub mod inbox_enums;
pub mod person;
pub mod post;

pub async fn shared_inbox(
  request: HttpRequest,
  input: web::Json<LemmyActivity<SharedInboxActivities>>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  receive_activity(request, input.into_inner(), None, context).await
}

async fn receive_activity<T>(
  request: HttpRequest,
  activity: LemmyActivity<T>,
  expected_name: Option<String>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError>
where
  T: ActivityHandler<Actor = lemmy_apub::fetcher::Actor>
    + Clone
    + Serialize
    + std::fmt::Debug
    + Send
    + 'static,
{
  // TODO: which order to check things?
  // Do nothing if we received the same activity before
  if is_activity_already_known(context.pool(), &activity.id_unchecked()).await? {
    return Ok(HttpResponse::Ok().finish());
  }
  assert_activity_not_local(&activity)?;
  check_is_apub_id_valid(&activity.actor, false)?;
  activity.inner.verify(&context).await?;

  let request_counter = &mut 0;
  let actor: Actor =
    get_or_fetch_and_upsert_actor(&activity.actor, &context, request_counter).await?;
  if let Some(expected) = expected_name {
    if expected != actor.name() {
      return Ok(HttpResponse::BadRequest().finish());
    }
  }
  verify_signature(&request, &actor.public_key().context(location_info!())?)?;

  // Log the activity, so we avoid receiving and parsing it twice. Note that this could still happen
  // if we receive the same activity twice in very quick succession.
  insert_activity(
    &activity.id_unchecked(),
    activity.clone(),
    false,
    true,
    context.pool(),
  )
  .await?;

  activity
    .inner
    .receive(actor, &context, request_counter)
    .await?;
  Ok(HttpResponse::Ok().finish())
}

/// Convert the data to json and turn it into an HTTP Response with the correct ActivityPub
/// headers.
fn create_apub_response<T>(data: &T) -> HttpResponse<Body>
where
  T: Serialize,
{
  HttpResponse::Ok()
    .content_type(APUB_JSON_CONTENT_TYPE)
    .json(data)
}

fn create_apub_tombstone_response<T>(data: &T) -> HttpResponse<Body>
where
  T: Serialize,
{
  HttpResponse::Gone()
    .content_type(APUB_JSON_CONTENT_TYPE)
    .status(StatusCode::GONE)
    .json(data)
}

#[derive(Deserialize)]
pub struct ActivityQuery {
  type_: String,
  id: String,
}

/// Return the ActivityPub json representation of a local activity over HTTP.
pub(crate) async fn get_activity(
  info: web::Path<ActivityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let settings = Settings::get();
  let activity_id = Url::parse(&format!(
    "{}/activities/{}/{}",
    settings.get_protocol_and_hostname(),
    info.type_,
    info.id
  ))?
  .into();
  let activity = blocking(context.pool(), move |conn| {
    Activity::read_from_apub_id(&conn, &activity_id)
  })
  .await??;

  let sensitive = activity.sensitive.unwrap_or(true);
  if !activity.local || sensitive {
    Ok(HttpResponse::NotFound().finish())
  } else {
    Ok(create_apub_response(&activity.data))
  }
}

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

pub(in crate::http) fn assert_activity_not_local<T: Debug>(
  activity: &LemmyActivity<T>,
) -> Result<(), LemmyError> {
  let activity_domain = activity.id_unchecked().domain().context(location_info!())?;

  if activity_domain == Settings::get().hostname() {
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
