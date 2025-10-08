use activitypub_federation::{
  actix_web::{
    inbox::{receive_activity_with_hook, ReceiveActivityHook},
    response::create_http_response,
    signing_actor,
  },
  config::Data,
  traits::{Activity, Object},
};
use actix_web::{
  web::{self, Bytes},
  HttpRequest,
  HttpResponse,
};
use either::Either;
use lemmy_api_utils::{context::LemmyContext, plugins::plugin_hook_after};
use lemmy_apub_activities::activity_lists::SharedInboxActivities;
use lemmy_apub_objects::objects::{SiteOrMultiOrCommunityOrUser, UserOrCommunity};
use lemmy_db_schema::{
  newtypes::InstanceId,
  source::{
    activity::{ReceivedActivity, SentActivity},
    community::Community,
  },
};
use lemmy_db_schema_file::enums::CommunityVisibility;
use lemmy_db_views_community_follower::CommunityFollowerView;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult, UntranslatedError},
  FEDERATION_CONTEXT,
};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::timeout;
use tracing::debug;
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
    receive_activity_with_hook::<SharedInboxActivities, UserOrCommunity, LemmyContext>(
      request, body, Dummy, &data,
    );
  // Set a timeout shorter than `REQWEST_TIMEOUT` for processing incoming activities. This is to
  // avoid taking a long time to process an incoming activity when a required data fetch times out.
  // In this case our own instance would timeout and be marked as dead by the sender. Better to
  // consider the activity broken and move on.
  timeout(INCOMING_ACTIVITY_TIMEOUT, receive_fut)
    .await
    .with_lemmy_type(UntranslatedError::InboxTimeout.into())?
}

struct Dummy;

impl ReceiveActivityHook<SharedInboxActivities, UserOrCommunity, LemmyContext> for Dummy {
  async fn hook(
    self,
    activity: &SharedInboxActivities,
    _actor: &UserOrCommunity,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    // Store received activities in the database. This ensures that the same activity doesn't get
    // received and processed more than once, which would be a waste of resources.
    debug!("Received activity {}", activity.id().to_string());
    ReceivedActivity::create(&mut context.pool(), &activity.id().clone().into()).await?;

    // This could also take the actor as param, but lifetimes and serde derives are tricky.
    // It is really a before hook, but doesnt allow modifying the data. It could use a
    // separate method so that error in plugin causes activity to be rejected.
    plugin_hook_after("activity_received", activity)?;

    // This method could also be used to check if actor is banned, instead of checking in each
    // activity handler.
    Ok(())
  }
}

#[derive(Deserialize)]
struct ActivityQuery {
  type_: String,
  id: String,
}

/// Return the ActivityPub json representation of a local activity over HTTP.
async fn get_activity(
  info: web::Path<ActivityQuery>,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
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
    Ok(create_http_response(&activity.data, &FEDERATION_CONTEXT)?)
  }
}

/// Ensure that the community is public and not removed/deleted.
fn check_community_fetchable(community: &Community) -> LemmyResult<()> {
  if !community.visibility.can_federate() {
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
  match community.visibility {
    Public | Unlisted => Ok(()),
    Private => {
      let signing_actor =
        signing_actor::<SiteOrMultiOrCommunityOrUser>(request, None, context).await?;
      if community.local {
        Ok(
          CommunityFollowerView::check_has_followers_from_instance(
            community.id,
            get_instance_id(&signing_actor),
            &mut context.pool(),
          )
          .await?,
        )
      } else if let Some(followers_url) = community.followers_url.clone() {
        let mut followers_url = followers_url.inner().clone();
        followers_url
          .query_pairs_mut()
          .append_pair("is_follower", signing_actor.id().as_str());
        let req = context.client().get(followers_url.as_str());
        let req = context.sign_request(req, Bytes::new()).await?;
        context.client().execute(req).await?.error_for_status()?;
        Ok(())
      } else {
        Err(LemmyErrorType::NotFound.into())
      }
    }
    LocalOnlyPublic | LocalOnlyPrivate => Err(LemmyErrorType::NotFound.into()),
  }
}

pub(in crate::http) fn get_instance_id(s: &SiteOrMultiOrCommunityOrUser) -> InstanceId {
  use Either::*;
  match s {
    Left(Left(s)) => s.instance_id,
    Left(Right(m)) => m.instance_id,
    Right(Left(u)) => u.instance_id,
    Right(Right(c)) => c.instance_id,
  }
}
