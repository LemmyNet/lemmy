use crate::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::{
    create_or_update::{note::CreateOrUpdateNote, page::CreateOrUpdatePage},
    CreateOrUpdateType,
  },
  CONTEXT,
};
use activitypub_federation::{
  activity_queue::send_activity,
  config::Data,
  fetch::object_id::ObjectId,
  kinds::public,
  protocol::context::WithContext,
  traits::{ActivityHandler, Actor},
};
use anyhow::anyhow;
use lemmy_api_common::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  newtypes::CommunityId,
  source::{
    activity::{SentActivity, SentActivityForm},
    community::Community,
    instance::Instance,
  },
};
use lemmy_db_views_actor::structs::{CommunityPersonBanView, CommunityView};
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult},
  spawn_try_task,
  SYNCHRONOUS_FEDERATION,
};
use moka::future::Cache;
use once_cell::sync::Lazy;
use serde::Serialize;
use std::{ops::Deref, sync::Arc, time::Duration};
use tracing::info;
use url::{ParseError, Url};
use uuid::Uuid;

pub mod block;
pub mod community;
pub mod create_or_update;
pub mod deletion;
pub mod following;
pub mod unfederated;
pub mod voting;

/// Amount of time that the list of dead instances is cached. This is only updated once a day,
/// so there is no harm in caching it for a longer time.
pub static DEAD_INSTANCE_LIST_CACHE_DURATION: Duration = Duration::from_secs(30 * 60);

/// Checks that the specified Url actually identifies a Person (by fetching it), and that the person
/// doesn't have a site ban.
#[tracing::instrument(skip_all)]
async fn verify_person(
  person_id: &ObjectId<ApubPerson>,
  context: &Data<LemmyContext>,
) -> Result<(), LemmyError> {
  let person = person_id.dereference(context).await?;
  if person.banned {
    return Err(anyhow!("Person {} is banned", person_id))
      .with_lemmy_type(LemmyErrorType::CouldntUpdateComment);
  }
  Ok(())
}

/// Fetches the person and community to verify their type, then checks if person is banned from site
/// or community.
#[tracing::instrument(skip_all)]
pub(crate) async fn verify_person_in_community(
  person_id: &ObjectId<ApubPerson>,
  community: &ApubCommunity,
  context: &Data<LemmyContext>,
) -> Result<(), LemmyError> {
  let person = person_id.dereference(context).await?;
  if person.banned {
    return Err(LemmyErrorType::PersonIsBannedFromSite)?;
  }
  let person_id = person.id;
  let community_id = community.id;
  let is_banned = CommunityPersonBanView::get(&mut context.pool(), person_id, community_id)
    .await
    .is_ok();
  if is_banned {
    return Err(LemmyErrorType::PersonIsBannedFromCommunity)?;
  }

  Ok(())
}

/// Verify that mod action in community was performed by a moderator.
///
/// * `mod_id` - Activitypub ID of the mod or admin who performed the action
/// * `object_id` - Activitypub ID of the actor or object that is being moderated
/// * `community` - The community inside which moderation is happening
#[tracing::instrument(skip_all)]
pub(crate) async fn verify_mod_action(
  mod_id: &ObjectId<ApubPerson>,
  object_id: &Url,
  community_id: CommunityId,
  context: &Data<LemmyContext>,
) -> Result<(), LemmyError> {
  let mod_ = mod_id.dereference(context).await?;

  let is_mod_or_admin =
    CommunityView::is_mod_or_admin(&mut context.pool(), mod_.id, community_id).await?;
  if is_mod_or_admin {
    return Ok(());
  }

  // mod action comes from the same instance as the moderated object, so it was presumably done
  // by an instance admin.
  // TODO: federate instance admin status and check it here
  if mod_id.inner().domain() == object_id.domain() {
    return Ok(());
  }

  Err(LemmyErrorType::NotAModerator)?
}

pub(crate) fn verify_is_public(to: &[Url], cc: &[Url]) -> Result<(), LemmyError> {
  if ![to, cc].iter().any(|set| set.contains(&public())) {
    return Err(LemmyErrorType::ObjectIsNotPublic)?;
  }
  Ok(())
}

pub(crate) fn verify_community_matches<T>(
  a: &ObjectId<ApubCommunity>,
  b: T,
) -> Result<(), LemmyError>
where
  T: Into<ObjectId<ApubCommunity>>,
{
  let b: ObjectId<ApubCommunity> = b.into();
  if a != &b {
    return Err(LemmyErrorType::InvalidCommunity)?;
  }
  Ok(())
}

pub(crate) fn check_community_deleted_or_removed(community: &Community) -> Result<(), LemmyError> {
  if community.deleted || community.removed {
    Err(LemmyErrorType::CannotCreatePostOrCommentInDeletedOrRemovedCommunity)?
  } else {
    Ok(())
  }
}

/// Generate a unique ID for an activity, in the format:
/// `http(s)://example.com/receive/create/202daf0a-1489-45df-8d2e-c8a3173fed36`
fn generate_activity_id<T>(kind: T, protocol_and_hostname: &str) -> Result<Url, ParseError>
where
  T: ToString,
{
  let id = format!(
    "{}/activities/{}/{}",
    protocol_and_hostname,
    kind.to_string().to_lowercase(),
    Uuid::new_v4()
  );
  Url::parse(&id)
}

#[tracing::instrument(skip_all)]
async fn send_lemmy_activity<Activity, ActorT>(
  data: &Data<LemmyContext>,
  activity: Activity,
  actor: &ActorT,
  mut inbox: Vec<Url>,
  sensitive: bool,
) -> Result<(), LemmyError>
where
  Activity: ActivityHandler + Serialize + Send + Sync + Clone,
  ActorT: Actor,
  Activity: ActivityHandler<Error = LemmyError>,
{
  static CACHE: Lazy<Cache<(), Arc<Vec<String>>>> = Lazy::new(|| {
    Cache::builder()
      .max_capacity(1)
      .time_to_live(DEAD_INSTANCE_LIST_CACHE_DURATION)
      .build()
  });
  let dead_instances = CACHE
    .try_get_with((), async {
      Ok::<_, diesel::result::Error>(Arc::new(Instance::dead_instances(&mut data.pool()).await?))
    })
    .await?;

  inbox.retain(|i| {
    let domain = i.domain().expect("has domain").to_string();
    !dead_instances.contains(&domain)
  });
  info!("Sending activity {}", activity.id().to_string());
  let activity = WithContext::new(activity, CONTEXT.deref().clone());

  let form = SentActivityForm {
    ap_id: activity.id().clone().into(),
    data: serde_json::to_value(activity.clone())?,
    sensitive,
  };
  SentActivity::create(&mut data.pool(), form).await?;
  send_activity(activity, actor, inbox, data).await?;

  Ok(())
}

pub async fn handle_outgoing_activities(context: Data<LemmyContext>) -> LemmyResult<()> {
  while let Some(data) = ActivityChannel::retrieve_activity().await {
    match_outgoing_activities(data, &context.reset_request_count()).await?
  }
  Ok(())
}

pub async fn match_outgoing_activities(
  data: SendActivityData,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let context = context.reset_request_count();
  let fed_task = async {
    match data {
      SendActivityData::CreatePost(post) => {
        let creator_id = post.creator_id;
        CreateOrUpdatePage::send(post, creator_id, CreateOrUpdateType::Create, context).await
      }
      SendActivityData::CreateComment(comment) => {
        let creator_id = comment.creator_id;
        CreateOrUpdateNote::send(comment, creator_id, CreateOrUpdateType::Create, context).await
      }
    }
  };
  if *SYNCHRONOUS_FEDERATION {
    fed_task.await?;
  } else {
    spawn_try_task(fed_task);
  }
  Ok(())
}
