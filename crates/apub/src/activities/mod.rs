use self::following::send_follow_community;
use crate::{
  activities::{
    block::{send_ban_from_community, send_ban_from_site},
    community::{
      collection_add::{send_add_mod_to_community, send_feature_post},
      lock_page::send_lock_post,
      update::send_update_community,
    },
    create_or_update::private_message::send_create_or_update_pm,
    deletion::{
      send_apub_delete_in_community,
      send_apub_delete_private_message,
      send_apub_delete_user,
      DeletableObjects,
    },
    voting::send_like_activity,
  },
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::{
    community::report::Report,
    create_or_update::{note::CreateOrUpdateNote, page::CreateOrUpdatePage},
    CreateOrUpdateType,
  },
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::{activity::AnnounceType, public},
  traits::{ActivityHandler, Actor},
};
use anyhow::anyhow;
use lemmy_api_common::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::{
  source::{
    activity::{ActivitySendTargets, ActorType, SentActivity, SentActivityForm},
    community::Community,
  },
  traits::Crud,
};
use lemmy_db_views_actor::structs::{CommunityPersonBanView, CommunityView};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult};
use serde::Serialize;
use tracing::info;
use url::{ParseError, Url};
use uuid::Uuid;

pub mod block;
pub mod community;
pub mod create_or_update;
pub mod deletion;
pub mod following;
pub mod voting;

/// Checks that the specified Url actually identifies a Person (by fetching it), and that the person
/// doesn't have a site ban.
#[tracing::instrument(skip_all)]
async fn verify_person(
  person_id: &ObjectId<ApubPerson>,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let person = person_id.dereference(context).await?;
  if person.banned {
    Err(anyhow!("Person {} is banned", person_id))
      .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)
  } else {
    Ok(())
  }
}

/// Fetches the person and community to verify their type, then checks if person is banned from site
/// or community.
#[tracing::instrument(skip_all)]
pub(crate) async fn verify_person_in_community(
  person_id: &ObjectId<ApubPerson>,
  community: &ApubCommunity,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let person = person_id.dereference(context).await?;
  if person.banned {
    Err(LemmyErrorType::PersonIsBannedFromSite(
      person.actor_id.to_string(),
    ))?
  }
  let person_id = person.id;
  let community_id = community.id;
  let is_banned = CommunityPersonBanView::get(&mut context.pool(), person_id, community_id).await?;
  if is_banned {
    Err(LemmyErrorType::PersonIsBannedFromCommunity)?
  } else {
    Ok(())
  }
}

/// Verify that mod action in community was performed by a moderator.
///
/// * `mod_id` - Activitypub ID of the mod or admin who performed the action
/// * `object_id` - Activitypub ID of the actor or object that is being moderated
/// * `community` - The community inside which moderation is happening
#[tracing::instrument(skip_all)]
pub(crate) async fn verify_mod_action(
  mod_id: &ObjectId<ApubPerson>,
  community: &Community,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let mod_ = mod_id.dereference(context).await?;

  let is_mod_or_admin =
    CommunityView::is_mod_or_admin(&mut context.pool(), mod_.id, community.id).await?;
  if is_mod_or_admin {
    return Ok(());
  }

  // mod action comes from the same instance as the community, so it was presumably done
  // by an instance admin.
  // TODO: federate instance admin status and check it here
  if mod_id.inner().domain() == community.actor_id.domain() {
    return Ok(());
  }

  Err(LemmyErrorType::NotAModerator)?
}

pub(crate) fn verify_is_public(to: &[Url], cc: &[Url]) -> LemmyResult<()> {
  if ![to, cc].iter().any(|set| set.contains(&public())) {
    Err(LemmyErrorType::ObjectIsNotPublic)?
  } else {
    Ok(())
  }
}

pub(crate) fn verify_community_matches<T>(a: &ObjectId<ApubCommunity>, b: T) -> LemmyResult<()>
where
  T: Into<ObjectId<ApubCommunity>>,
{
  let b: ObjectId<ApubCommunity> = b.into();
  if a != &b {
    Err(LemmyErrorType::InvalidCommunity)?
  } else {
    Ok(())
  }
}

pub(crate) fn check_community_deleted_or_removed(community: &Community) -> LemmyResult<()> {
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

/// like generate_activity_id but also add the inner kind for easier debugging
fn generate_announce_activity_id(
  inner_kind: &str,
  protocol_and_hostname: &str,
) -> Result<Url, ParseError> {
  let id = format!(
    "{}/activities/{}/{}/{}",
    protocol_and_hostname,
    AnnounceType::Announce.to_string().to_lowercase(),
    inner_kind.to_lowercase(),
    Uuid::new_v4()
  );
  Url::parse(&id)
}

pub(crate) trait GetActorType {
  fn actor_type(&self) -> ActorType;
}

#[tracing::instrument(skip_all)]
async fn send_lemmy_activity<Activity, ActorT>(
  data: &Data<LemmyContext>,
  activity: Activity,
  actor: &ActorT,
  send_targets: ActivitySendTargets,
  sensitive: bool,
) -> LemmyResult<()>
where
  Activity: ActivityHandler + Serialize + Send + Sync + Clone,
  ActorT: Actor + GetActorType,
  Activity: ActivityHandler<Error = LemmyError>,
{
  info!("Saving outgoing activity to queue {}", activity.id());

  let form = SentActivityForm {
    ap_id: activity.id().clone().into(),
    data: serde_json::to_value(activity)?,
    sensitive,
    send_inboxes: send_targets
      .inboxes
      .into_iter()
      .map(|e| Some(e.into()))
      .collect(),
    send_all_instances: send_targets.all_instances,
    send_community_followers_of: send_targets.community_followers_of.map(|e| e.0),
    actor_type: actor.actor_type(),
    actor_apub_id: actor.id().into(),
  };
  SentActivity::create(&mut data.pool(), form).await?;

  Ok(())
}

pub async fn handle_outgoing_activities(context: Data<LemmyContext>) {
  while let Some(data) = ActivityChannel::retrieve_activity().await {
    if let Err(e) = match_outgoing_activities(data, &context.reset_request_count()).await {
      tracing::warn!("error while saving outgoing activity to db: {e}");
    }
  }
}

pub async fn match_outgoing_activities(
  data: SendActivityData,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let context = context.reset_request_count();
  let fed_task = async {
    use SendActivityData::*;
    match data {
      CreatePost(post) => {
        let creator_id = post.creator_id;
        CreateOrUpdatePage::send(post, creator_id, CreateOrUpdateType::Create, context).await
      }
      UpdatePost(post) => {
        let creator_id = post.creator_id;
        CreateOrUpdatePage::send(post, creator_id, CreateOrUpdateType::Update, context).await
      }
      DeletePost(post, person, data) => {
        let community = Community::read(&mut context.pool(), post.community_id).await?;
        send_apub_delete_in_community(
          person,
          community,
          DeletableObjects::Post(post.into()),
          None,
          data.deleted,
          &context,
        )
        .await
      }
      RemovePost {
        post,
        moderator,
        reason,
        removed,
      } => {
        let community = Community::read(&mut context.pool(), post.community_id).await?;
        send_apub_delete_in_community(
          moderator,
          community,
          DeletableObjects::Post(post.into()),
          reason.or_else(|| Some(String::new())),
          removed,
          &context,
        )
        .await
      }
      LockPost(post, actor, locked) => send_lock_post(post, actor, locked, context).await,
      FeaturePost(post, actor, featured) => send_feature_post(post, actor, featured, context).await,
      CreateComment(comment) => {
        let creator_id = comment.creator_id;
        CreateOrUpdateNote::send(comment, creator_id, CreateOrUpdateType::Create, context).await
      }
      UpdateComment(comment) => {
        let creator_id = comment.creator_id;
        CreateOrUpdateNote::send(comment, creator_id, CreateOrUpdateType::Update, context).await
      }
      DeleteComment(comment, actor, community) => {
        let is_deleted = comment.deleted;
        let deletable = DeletableObjects::Comment(comment.into());
        send_apub_delete_in_community(actor, community, deletable, None, is_deleted, &context).await
      }
      RemoveComment {
        comment,
        moderator,
        community,
        reason,
      } => {
        let is_removed = comment.removed;
        let deletable = DeletableObjects::Comment(comment.into());
        send_apub_delete_in_community(
          moderator, community, deletable, reason, is_removed, &context,
        )
        .await
      }
      LikePostOrComment {
        object_id,
        actor,
        community,
        score,
      } => send_like_activity(object_id, actor, community, score, context).await,
      FollowCommunity(community, person, follow) => {
        send_follow_community(community, person, follow, &context).await
      }
      UpdateCommunity(actor, community) => send_update_community(community, actor, context).await,
      DeleteCommunity(actor, community, removed) => {
        let deletable = DeletableObjects::Community(community.clone().into());
        send_apub_delete_in_community(actor, community, deletable, None, removed, &context).await
      }
      RemoveCommunity {
        moderator,
        community,
        reason,
        removed,
      } => {
        let deletable = DeletableObjects::Community(community.clone().into());
        send_apub_delete_in_community(
          moderator,
          community,
          deletable,
          reason.clone().or_else(|| Some(String::new())),
          removed,
          &context,
        )
        .await
      }
      AddModToCommunity {
        moderator,
        community_id,
        target,
        added,
      } => send_add_mod_to_community(moderator, community_id, target, added, context).await,
      BanFromCommunity {
        moderator,
        community_id,
        target,
        data,
      } => send_ban_from_community(moderator, community_id, target, data, context).await,
      BanFromSite {
        moderator,
        banned_user,
        reason,
        remove_data,
        ban,
        expires,
      } => {
        send_ban_from_site(
          moderator,
          banned_user,
          reason,
          remove_data,
          ban,
          expires,
          context,
        )
        .await
      }
      CreatePrivateMessage(pm) => {
        send_create_or_update_pm(pm, CreateOrUpdateType::Create, context).await
      }
      UpdatePrivateMessage(pm) => {
        send_create_or_update_pm(pm, CreateOrUpdateType::Update, context).await
      }
      DeletePrivateMessage(person, pm, deleted) => {
        send_apub_delete_private_message(&person.into(), pm, deleted, context).await
      }
      DeleteUser(person, remove_data) => send_apub_delete_user(person, remove_data, context).await,
      CreateReport {
        object_id,
        actor,
        community,
        reason,
      } => Report::send(ObjectId::from(object_id), actor, community, reason, context).await,
    }
  };
  fed_task.await?;
  Ok(())
}
