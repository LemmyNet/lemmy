use crate::{
  activity_lists::SharedInboxActivities,
  check_is_apub_id_valid,
  context::{WithContext, WithContextJson},
  fetcher::user_or_community::UserOrCommunity,
  http::{community::receive_group_inbox, person::receive_person_inbox},
  insert_activity,
};
use actix_web::{
  body::Body,
  web,
  web::{Bytes, BytesMut, Payload},
  HttpRequest,
  HttpResponse,
};
use anyhow::{anyhow, Context};
use futures::StreamExt;
use http::StatusCode;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  signatures::verify_signature,
  traits::{ActivityHandler, ActorType},
  APUB_JSON_CONTENT_TYPE,
};
use lemmy_db_schema::{source::activity::Activity, DbPool};
use lemmy_utils::{location_info, LemmyError};
use lemmy_websocket::LemmyContext;
use log::info;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, io::Read};
use url::Url;

mod comment;
mod community;
mod person;
mod post;
pub mod routes;

pub async fn shared_inbox(
  request: HttpRequest,
  payload: Payload,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let unparsed = payload_to_string(payload).await?;
  info!("Received shared inbox activity {}", unparsed);
  let activity_data: ActivityCommonFields = serde_json::from_str(&unparsed)?;
  let activity = serde_json::from_str::<WithContext<SharedInboxActivities>>(&unparsed)?;
  match activity.inner() {
    SharedInboxActivities::GroupInboxActivities(g) => {
      receive_group_inbox(*g, activity_data, request, &context).await
    }
    SharedInboxActivities::PersonInboxActivities(p) => {
      receive_person_inbox(*p, activity_data, request, &context).await
    }
  }
}

async fn payload_to_string(mut payload: Payload) -> Result<String, LemmyError> {
  let mut bytes = BytesMut::new();
  while let Some(item) = payload.next().await {
    bytes.extend_from_slice(&item?);
  }
  let mut unparsed = String::new();
  Bytes::from(bytes).as_ref().read_to_string(&mut unparsed)?;
  Ok(unparsed)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActivityCommonFields {
  pub(crate) id: Url,
  pub(crate) actor: Url,
}

// TODO: move most of this code to library
async fn receive_activity<'a, T>(
  request: HttpRequest,
  activity: T,
  activity_data: ActivityCommonFields,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError>
where
  T: ActivityHandler<DataType = LemmyContext>
    + Clone
    + Deserialize<'a>
    + Serialize
    + std::fmt::Debug
    + Send
    + 'static,
{
  check_is_apub_id_valid(&activity_data.actor, false, &context.settings())?;
  let request_counter = &mut 0;
  let actor = ObjectId::<UserOrCommunity>::new(activity_data.actor)
    .dereference(context, request_counter)
    .await?;
  verify_signature(&request, &actor.public_key().context(location_info!())?)?;

  // Do nothing if we received the same activity before
  if is_activity_already_known(context.pool(), &activity_data.id).await? {
    return Ok(HttpResponse::Ok().finish());
  }
  info!("Verifying activity {}", activity_data.id.to_string());
  activity
    .verify(&Data::new(context.clone()), request_counter)
    .await?;
  assert_activity_not_local(&activity_data.id, &context.settings().hostname)?;

  // Log the activity, so we avoid receiving and parsing it twice. Note that this could still happen
  // if we receive the same activity twice in very quick succession.
  insert_activity(&activity_data.id, &activity, false, true, context.pool()).await?;

  info!("Receiving activity {}", activity_data.id.to_string());
  activity
    .receive(&Data::new(context.clone()), request_counter)
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
    .json(WithContext::new(data))
}

fn create_json_apub_response(data: serde_json::Value) -> HttpResponse<Body> {
  HttpResponse::Ok()
    .content_type(APUB_JSON_CONTENT_TYPE)
    .json(WithContextJson::new(data))
}

fn create_apub_tombstone_response<T>(data: &T) -> HttpResponse<Body>
where
  T: Serialize,
{
  HttpResponse::Gone()
    .content_type(APUB_JSON_CONTENT_TYPE)
    .status(StatusCode::GONE)
    .json(WithContext::new(data))
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

pub(crate) async fn is_activity_already_known(
  pool: &DbPool,
  activity_id: &Url,
) -> Result<bool, LemmyError> {
  let activity_id = activity_id.to_owned().into();
  let existing = blocking(pool, move |conn| {
    Activity::read_from_apub_id(conn, &activity_id)
  })
  .await?;
  match existing {
    Ok(_) => Ok(true),
    Err(_) => Ok(false),
  }
}

fn assert_activity_not_local(id: &Url, hostname: &str) -> Result<(), LemmyError> {
  let activity_domain = id.domain().context(location_info!())?;

  if activity_domain == hostname {
    return Err(
      anyhow!(
        "Error: received activity which was sent by local instance: {:?}",
        id
      )
      .into(),
    );
  }
  Ok(())
}
