use crate::{
  activity_lists::SharedInboxActivities,
  fetcher::user_or_community::UserOrCommunity,
  protocol::objects::tombstone::Tombstone,
  CONTEXT,
};
use activitypub_federation::{
  actix_web::inbox::receive_activity,
  config::Data,
  protocol::context::WithContext,
  FEDERATION_CONTENT_TYPE,
};
use actix_web::{web, web::Bytes, HttpRequest, HttpResponse};
use http::StatusCode;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::activity::Activity;
use lemmy_utils::error::LemmyError;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use url::Url;

mod comment;
mod community;
mod person;
mod post;
pub mod routes;
pub mod site;

pub async fn shared_inbox(
  request: HttpRequest,
  body: Bytes,
  data: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  receive_activity::<SharedInboxActivities, UserOrCommunity, LemmyContext>(request, body, &data)
    .await
}

/// Convert the data to json and turn it into an HTTP Response with the correct ActivityPub
/// headers.
fn create_apub_response<T>(data: &T) -> HttpResponse
where
  T: Serialize,
{
  HttpResponse::Ok()
    .content_type(FEDERATION_CONTENT_TYPE)
    .json(WithContext::new(data, CONTEXT.deref().clone()))
}

fn create_json_apub_response(data: serde_json::Value) -> HttpResponse {
  HttpResponse::Ok()
    .content_type(FEDERATION_CONTENT_TYPE)
    .json(data)
}

fn create_apub_tombstone_response<T: Into<Url>>(id: T) -> HttpResponse {
  let tombstone = Tombstone::new(id.into());
  HttpResponse::Gone()
    .content_type(FEDERATION_CONTENT_TYPE)
    .status(StatusCode::GONE)
    .json(WithContext::new(tombstone, CONTEXT.deref().clone()))
}

fn err_object_not_local() -> LemmyError {
  LemmyError::from_message("Object not local, fetch it from original instance")
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
  let activity = Activity::read_from_apub_id(context.pool(), &activity_id).await?;

  let sensitive = activity.sensitive.unwrap_or(true);
  if !activity.local {
    Err(err_object_not_local())
  } else if sensitive {
    Ok(HttpResponse::Forbidden().finish())
  } else {
    Ok(create_json_apub_response(activity.data))
  }
}
