use crate::{
  activity_lists::SharedInboxActivities,
  fetcher::{site_or_community_or_user::SiteOrCommunityOrUser, user_or_community::UserOrCommunity},
  protocol::objects::tombstone::Tombstone,
  FEDERATION_CONTEXT,
};
use activitypub_federation::{
  actix_web::{inbox::receive_activity, signing_actor},
  config::Data,
  protocol::context::WithContext,
  FEDERATION_CONTENT_TYPE,
};
use actix_web::{web, web::Bytes, HttpRequest, HttpResponse};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  newtypes::DbUrl,
  source::{activity::SentActivity, community::Community},
  CommunityVisibility,
};
use lemmy_db_views_actor::structs::CommunityFollowerView;
use lemmy_utils::error::{FederationError, LemmyErrorExt, LemmyErrorType, LemmyResult};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, time::Duration};
use tokio::time::timeout;
use url::Url;

mod comment;
mod community;
mod person;
mod post;
pub mod routes;
pub mod site;

const INCOMING_ACTIVITY_TIMEOUT: Duration = Duration::from_secs(9);

pub async fn shared_inbox(
  request: HttpRequest,
  body: Bytes,
  data: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let receive_fut =
    receive_activity::<SharedInboxActivities, UserOrCommunity, LemmyContext>(request, body, &data);
  // Set a timeout shorter than `REQWEST_TIMEOUT` for processing incoming activities. This is to
  // avoid taking a long time to process an incoming activity when a required data fetch times out.
  // In this case our own instance would timeout and be marked as dead by the sender. Better to
  // consider the activity broken and move on.
  timeout(INCOMING_ACTIVITY_TIMEOUT, receive_fut)
    .await
    .with_lemmy_type(FederationError::InboxTimeout.into())?
}

/// Convert the data to json and turn it into an HTTP Response with the correct ActivityPub
/// headers.
///
/// actix-web doesn't allow pretty-print for json so we need to do this manually.
fn create_apub_response<T>(data: &T) -> LemmyResult<HttpResponse>
where
  T: Serialize,
{
  let json = serde_json::to_string_pretty(&WithContext::new(data, FEDERATION_CONTEXT.clone()))?;

  Ok(
    HttpResponse::Ok()
      .content_type(FEDERATION_CONTENT_TYPE)
      .body(json),
  )
}

fn create_apub_tombstone_response<T: Into<Url>>(id: T) -> LemmyResult<HttpResponse> {
  let tombstone = Tombstone::new(id.into());
  let json = serde_json::to_string_pretty(&WithContext::new(
    tombstone,
    FEDERATION_CONTEXT.deref().clone(),
  ))?;

  Ok(
    HttpResponse::Gone()
      .content_type(FEDERATION_CONTENT_TYPE)
      .status(actix_web::http::StatusCode::GONE)
      .body(json),
  )
}

fn redirect_remote_object(url: &DbUrl) -> HttpResponse {
  let mut res = HttpResponse::PermanentRedirect();
  res.insert_header((actix_web::http::header::LOCATION, url.as_str()));
  res.finish()
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
) -> LemmyResult<HttpResponse> {
  let settings = context.settings();
  let activity_id = Url::parse(&format!(
    "{}/activities/{}/{}",
    settings.get_protocol_and_hostname(),
    info.type_,
    info.id
  ))?
  .into();
  let activity = SentActivity::read_from_apub_id(&mut context.pool(), &activity_id)
    .await
    .with_lemmy_type(FederationError::CouldntFindActivity.into())?;

  let sensitive = activity.sensitive;
  if sensitive {
    Ok(HttpResponse::Forbidden().finish())
  } else {
    create_apub_response(&activity.data)
  }
}

/// Ensure that the community is public and not removed/deleted.
fn check_community_fetchable(community: &Community) -> LemmyResult<()> {
  check_community_removed_or_deleted(community)?;
  if community.visibility == CommunityVisibility::LocalOnly {
    return Err(LemmyErrorType::NotFound.into());
  }
  Ok(())
}

/// Check if posts or comments in the community are allowed to be fetched
async fn check_community_content_fetchable(
  community: &Community,
  request: &HttpRequest,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  use CommunityVisibility::*;
  check_community_removed_or_deleted(community)?;
  match community.visibility {
    // content in public community can always be fetched
    Public => Ok(()),
    // no federation for local only community
    LocalOnly => Err(LemmyErrorType::NotFound.into()),
    // for private community check http signature of request, if there is any approved follower
    // from the fetching instance then fetching is allowed
    Private => {
      let signing_actor = signing_actor::<SiteOrCommunityOrUser>(request, None, context).await?;
      Ok(
        CommunityFollowerView::check_has_followers_from_instance(
          community.id,
          signing_actor.instance_id(),
          &mut context.pool(),
        )
        .await?,
      )
    }
  }
}

fn check_community_removed_or_deleted(community: &Community) -> LemmyResult<()> {
  if community.deleted || community.removed {
    Err(LemmyErrorType::Deleted)?
  }
  Ok(())
}
