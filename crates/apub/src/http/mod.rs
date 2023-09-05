use crate::{
    activity_lists::SharedInboxActivities, fetcher::user_or_community::UserOrCommunity,
    protocol::objects::tombstone::Tombstone, CONTEXT,
};
use activitypub_federation::{
    actix_web::inbox::receive_activity, config::Data, protocol::context::WithContext,
    FEDERATION_CONTENT_TYPE,
};
use actix_web::{web, web::Bytes, HttpRequest, HttpResponse};
use http::StatusCode;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::activity::SentActivity;
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
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
) -> LemmyResult<HttpResponse> {
    receive_activity::<SharedInboxActivities, UserOrCommunity, LemmyContext>(request, body, &data)
        .await
}

/// Convert the data to json and turn it into an HTTP Response with the correct ActivityPub
/// headers.
///
/// actix-web doesn't allow pretty-print for json so we need to do this manually.
fn create_apub_response<T>(data: &T) -> LemmyResult<HttpResponse>
where
    T: Serialize,
{
    let json = serde_json::to_string_pretty(&WithContext::new(data, CONTEXT.clone()))?;

    Ok(HttpResponse::Ok()
        .content_type(FEDERATION_CONTENT_TYPE)
        .body(json))
}

fn create_apub_tombstone_response<T: Into<Url>>(id: T) -> LemmyResult<HttpResponse> {
    let tombstone = Tombstone::new(id.into());
    let json = serde_json::to_string_pretty(&WithContext::new(tombstone, CONTEXT.deref().clone()))?;

    Ok(HttpResponse::Gone()
        .content_type(FEDERATION_CONTENT_TYPE)
        .status(StatusCode::GONE)
        .body(json))
}

fn err_object_not_local() -> LemmyError {
    LemmyErrorType::ObjectNotLocal.into()
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
    let activity = SentActivity::read_from_apub_id(&mut context.pool(), &activity_id).await?;

    let sensitive = activity.sensitive;
    if sensitive {
        Ok(HttpResponse::Forbidden().finish())
    } else {
        create_apub_response(&activity.data)
    }
}
