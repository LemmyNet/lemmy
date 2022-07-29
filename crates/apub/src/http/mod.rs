use crate::{
  activity_lists::SharedInboxActivities,
  fetcher::user_or_community::UserOrCommunity,
  insert_activity,
  local_instance,
  protocol::objects::tombstone::Tombstone,
  CONTEXT,
};
use activitypub_federation::{
  core::inbox::receive_activity,
  data::Data,
  deser::context::WithContext,
  traits::{ActivityHandler, Actor, ApubObject},
  APUB_JSON_CONTENT_TYPE,
};
use actix_web::{web, HttpRequest, HttpResponse};
use http::StatusCode;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::source::activity::Activity;
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::ops::Deref;
use tracing::{debug, log::info};
use url::Url;

mod comment;
mod community;
mod person;
mod post;
pub mod routes;
pub mod site;

#[tracing::instrument(skip_all)]
pub async fn shared_inbox(
  request: HttpRequest,
  payload: String,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  receive_lemmy_activity::<SharedInboxActivities, UserOrCommunity>(request, payload, context).await
}

pub async fn receive_lemmy_activity<Activity, ActorT>(
  request: HttpRequest,
  payload: String,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError>
where
  Activity: ActivityHandler<DataType = LemmyContext, Error = LemmyError>
    + DeserializeOwned
    + Send
    + 'static,
  ActorT: ApubObject<DataType = LemmyContext, Error = LemmyError> + Actor + Send + 'static,
  for<'de2> <ActorT as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
  let activity_value: Value = serde_json::from_str(&payload)?;
  debug!("Received activity {:#}", payload.as_str());
  let activity: Activity = serde_json::from_value(activity_value.clone())?;
  // Log the activity, so we avoid receiving and parsing it twice.
  let insert = insert_activity(activity.id(), activity_value, false, true, context.pool()).await?;
  if !insert {
    debug!("Received duplicate activity {}", activity.id().to_string());
    return Ok(HttpResponse::BadRequest().finish());
  }
  info!("Received activity {}", payload);

  static DATA: OnceCell<Data<LemmyContext>> = OnceCell::new();
  let data = DATA.get_or_init(|| Data::new(context.get_ref().clone()));
  receive_activity::<Activity, ActorT, LemmyContext>(
    request,
    activity,
    local_instance(&context),
    data,
  )
  .await
}

/// Convert the data to json and turn it into an HTTP Response with the correct ActivityPub
/// headers.
fn create_apub_response<T>(data: &T) -> HttpResponse
where
  T: Serialize,
{
  HttpResponse::Ok()
    .content_type(APUB_JSON_CONTENT_TYPE)
    .json(WithContext::new(data, CONTEXT.deref().clone()))
}

fn create_json_apub_response(data: serde_json::Value) -> HttpResponse {
  HttpResponse::Ok()
    .content_type(APUB_JSON_CONTENT_TYPE)
    .json(data)
}

fn create_apub_tombstone_response<T: Into<Url>>(id: T) -> HttpResponse {
  let tombstone = Tombstone::new(id.into());
  HttpResponse::Gone()
    .content_type(APUB_JSON_CONTENT_TYPE)
    .status(StatusCode::GONE)
    .json(WithContext::new(tombstone, CONTEXT.deref().clone()))
}

#[derive(Deserialize)]
pub struct ActivityQuery {
  type_: String,
  id: String,
}

/// Return the ActivityPub json representation of a local activity over HTTP.
#[tracing::instrument(skip_all)]
pub(crate) async fn get_activity(
  info: web::Path<ActivityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let settings = context.settings();
  let activity_id = Url::parse(&format!(
    "{}/activities/{}/{}",
    settings.get_protocol_and_hostname(),
    info.type_,
    info.id
  ))?
  .into();
  let activity = blocking(context.pool(), move |conn| {
    Activity::read_from_apub_id(conn, &activity_id)
  })
  .await??;

  let sensitive = activity.sensitive.unwrap_or(true);
  if !activity.local || sensitive {
    Ok(HttpResponse::NotFound().finish())
  } else {
    Ok(create_json_apub_response(activity.data))
  }
}
