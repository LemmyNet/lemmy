use crate::APUB_JSON_CONTENT_TYPE;
use actix_web::{body::Body, web, HttpResponse};
use http::StatusCode;
use lemmy_db_queries::source::activity::Activity_;
use lemmy_db_schema::source::activity::Activity;
use lemmy_structs::blocking;
use lemmy_utils::{settings::Settings, LemmyError};
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};

pub mod comment;
pub mod community;
pub mod post;
pub mod user;

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
pub struct CommunityQuery {
  type_: String,
  id: String,
}

/// Return the ActivityPub json representation of a local community over HTTP.
pub async fn get_activity(
  info: web::Path<CommunityQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let settings = Settings::get();
  let activity_id = format!(
    "{}/activities/{}/{}",
    settings.get_protocol_and_hostname(),
    info.type_,
    info.id
  );
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
