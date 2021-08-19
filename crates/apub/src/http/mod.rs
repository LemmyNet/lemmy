use crate::{
  check_is_apub_id_valid,
  extensions::signatures::verify_signature,
  fetcher::get_or_fetch_and_upsert_actor,
  http::{
    community::{receive_group_inbox, GroupInboxActivities},
    person::{receive_person_inbox, PersonInboxActivities},
  },
  insert_activity,
  APUB_JSON_CONTENT_TYPE,
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
use lemmy_apub_lib::{ActivityFields, ActivityHandler};
use lemmy_db_queries::{source::activity::Activity_, DbPool};
use lemmy_db_schema::source::activity::Activity;
use lemmy_utils::{location_info, settings::structs::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use log::{info, trace};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, io::Read};
use url::Url;

mod comment;
mod community;
mod person;
mod post;
pub mod routes;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityHandler, ActivityFields)]
#[serde(untagged)]
pub enum SharedInboxActivities {
  GroupInboxActivities(GroupInboxActivities),
  // Note, pm activities need to be at the end, otherwise comments will end up here. We can probably
  // avoid this problem by replacing createpm.object with our own struct, instead of NoteExt.
  PersonInboxActivities(PersonInboxActivities),
}

pub async fn shared_inbox(
  request: HttpRequest,
  payload: Payload,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let unparsed = payload_to_string(payload).await?;
  trace!("Received shared inbox activity {}", unparsed);
  let activity = serde_json::from_str::<SharedInboxActivities>(&unparsed)?;
  match activity {
    SharedInboxActivities::GroupInboxActivities(g) => {
      receive_group_inbox(g, request, &context).await
    }
    SharedInboxActivities::PersonInboxActivities(p) => {
      receive_person_inbox(p, request, &context).await
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

// TODO: move most of this code to library
async fn receive_activity<'a, T>(
  request: HttpRequest,
  activity: T,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError>
where
  T: ActivityHandler
    + ActivityFields
    + Clone
    + Deserialize<'a>
    + Serialize
    + std::fmt::Debug
    + Send
    + 'static,
{
  let request_counter = &mut 0;
  let actor = get_or_fetch_and_upsert_actor(activity.actor(), context, request_counter).await?;
  verify_signature(&request, &actor.public_key().context(location_info!())?)?;

  // Do nothing if we received the same activity before
  if is_activity_already_known(context.pool(), activity.id_unchecked()).await? {
    return Ok(HttpResponse::Ok().finish());
  }
  check_is_apub_id_valid(activity.actor(), false)?;
  info!("Verifying activity {}", activity.id_unchecked().to_string());
  activity.verify(context, request_counter).await?;
  assert_activity_not_local(&activity)?;

  // Log the activity, so we avoid receiving and parsing it twice. Note that this could still happen
  // if we receive the same activity twice in very quick succession.
  insert_activity(
    activity.id_unchecked(),
    activity.clone(),
    false,
    true,
    context.pool(),
  )
  .await?;

  info!("Receiving activity {}", activity.id_unchecked().to_string());
  activity.receive(context, request_counter).await?;
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
    Activity::read_from_apub_id(conn, &activity_id)
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
    Activity::read_from_apub_id(conn, &activity_id)
  })
  .await?;
  match existing {
    Ok(_) => Ok(true),
    Err(_) => Ok(false),
  }
}

fn assert_activity_not_local<T: Debug + ActivityFields>(activity: &T) -> Result<(), LemmyError> {
  let activity_domain = activity.id_unchecked().domain().context(location_info!())?;

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
