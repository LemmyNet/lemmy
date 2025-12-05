use crate::{
  block::{send_ban_from_community, send_ban_from_site},
  community::{
    collection_add::{send_add_mod_to_community, send_feature_post},
    lock::send_lock,
    update::{send_update_community, send_update_multi_community},
  },
  create_or_update::private_message::send_create_or_update_pm,
  deletion::{
    DeletableObjects,
    send_apub_delete_in_community,
    send_apub_delete_private_message,
    send_apub_delete_user,
  },
  following::send_follow,
  protocol::{
    CreateOrUpdateType,
    community::{report::Report, resolve_report::ResolveReport},
    create_or_update::{note::CreateOrUpdateNote, page::CreateOrUpdatePage},
  },
  voting::send_like_activity,
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::AnnounceType,
  traits::{Activity, Actor},
};
use either::Either;
use following::send_accept_or_reject_follow;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_apub_objects::{
  objects::{PostOrComment, person::ApubPerson},
  utils::functions::GetActorType,
};
use lemmy_db_schema::source::{
  activity::{ActivitySendTargets, SentActivity, SentActivityForm},
  community::Community,
  instance::InstanceActions,
};
use lemmy_db_views_post::PostView;
use lemmy_db_views_site::SiteView;
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::{LemmyError, LemmyResult, UntranslatedError};
use serde::Serialize;
use tracing::info;
use url::{ParseError, Url};
use uuid::Uuid;

pub mod activity_lists;
pub mod block;
pub mod community;
pub mod create_or_update;
pub mod deletion;
pub mod following;
pub mod protocol;
pub mod voting;

const MOD_ACTION_DEFAULT_REASON: &str = "No reason provided";

/// Checks that the specified Url actually identifies a Person (by fetching it), and that the person
/// doesn't have a site ban.
async fn verify_person(
  person_id: &ObjectId<ApubPerson>,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let person = person_id.dereference(context).await?;
  InstanceActions::check_ban(&mut context.pool(), person.id, person.instance_id).await?;
  Ok(())
}

pub(crate) fn check_community_deleted_or_removed(community: &Community) -> LemmyResult<()> {
  if community.deleted || community.removed {
    Err(UntranslatedError::CannotCreatePostOrCommentInDeletedOrRemovedCommunity)?
  } else {
    Ok(())
  }
}

/// Generate a unique ID for an activity, in the format:
/// `http(s)://example.com/receive/create/202daf0a-1489-45df-8d2e-c8a3173fed36`
fn generate_activity_id<T>(kind: T, context: &LemmyContext) -> Result<Url, ParseError>
where
  T: ToString,
{
  let id = format!(
    "{}/activities/{}/{}",
    &context.settings().get_protocol_and_hostname(),
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

async fn send_lemmy_activity<A, ActorT>(
  data: &Data<LemmyContext>,
  activity: A,
  actor: &ActorT,
  send_targets: ActivitySendTargets,
  sensitive: bool,
) -> LemmyResult<()>
where
  A: Activity + Serialize + Send + Sync + Clone + Activity<Error = LemmyError>,
  ActorT: Actor + GetActorType,
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
    actor_apub_id: actor.id().clone().into(),
  };
  SentActivity::create(&mut data.pool(), form).await?;

  Ok(())
}

pub async fn handle_outgoing_activities(context: Data<LemmyContext>) {
  while let Some(data) = ActivityChannel::retrieve_activity().await {
    if let Err(e) = match_outgoing_activities(data, &context).await {
      tracing::warn!("error while saving outgoing activity to db: {e}");
    }
  }
}

pub async fn match_outgoing_activities(
  data: SendActivityData,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let context = context.clone();
  Box::pin(async {
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
      DeletePost(post, person, community) => {
        let is_deleted = post.deleted;
        send_apub_delete_in_community(
          person,
          community,
          DeletableObjects::Post(post.into()),
          None,
          is_deleted,
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
          Some(reason),
          removed,
          &context,
        )
        .await
      }
      LockPost(post, actor, locked, reason) => {
        send_lock(
          PostOrComment::Left(post.into()),
          actor,
          locked,
          reason,
          context,
        )
        .await
      }
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
          moderator,
          community,
          deletable,
          Some(reason),
          is_removed,
          &context,
        )
        .await
      }
      LockComment(comment, actor, locked, reason) => {
        send_lock(
          PostOrComment::Right(comment.into()),
          actor,
          locked,
          reason,
          context,
        )
        .await
      }
      LikePostOrComment {
        object_id,
        actor,
        community,
        previous_is_upvote,
        new_is_upvote,
      } => {
        send_like_activity(
          object_id,
          actor,
          community,
          previous_is_upvote,
          new_is_upvote,
          context,
        )
        .await
      }
      FollowCommunity(community, person, follow) => {
        send_follow(Either::Left(community.into()), person, follow, &context).await
      }
      FollowMultiCommunity(multi, person, follow) => {
        send_follow(Either::Right(multi.into()), person, follow, &context).await
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
          Some(reason),
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
        remove_or_restore_data,
        ban,
        expires_at,
      } => {
        send_ban_from_site(
          moderator,
          banned_user,
          reason,
          remove_or_restore_data,
          ban,
          expires_at,
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
        receiver,
        reason,
      } => {
        Report::send(
          ObjectId::from(object_id),
          &actor.into(),
          &receiver.map_either(Into::into, Into::into),
          reason,
          context,
        )
        .await
      }
      SendResolveReport {
        object_id,
        actor,
        report_creator,
        receiver,
      } => {
        ResolveReport::send(
          ObjectId::from(object_id),
          &actor.into(),
          &report_creator.into(),
          &receiver.map_either(Into::into, Into::into),
          context,
        )
        .await
      }
      AcceptFollower(community_id, person_id) => {
        send_accept_or_reject_follow(community_id, person_id, true, &context).await
      }
      RejectFollower(community_id, person_id) => {
        send_accept_or_reject_follow(community_id, person_id, false, &context).await
      }
      UpdateMultiCommunity(multi, actor) => {
        send_update_multi_community(multi, actor, context).await
      }
    }
  })
  .await?;
  Ok(())
}

pub(crate) async fn post_or_comment_community(
  post_or_comment: &PostOrComment,
  context: &Data<LemmyContext>,
) -> LemmyResult<Community> {
  match post_or_comment {
    PostOrComment::Left(p) => Community::read(&mut context.pool(), p.community_id).await,
    PostOrComment::Right(c) => {
      let site_view = SiteView::read_local(&mut context.pool()).await?;
      Ok(
        PostView::read(
          &mut context.pool(),
          c.post_id,
          None,
          site_view.instance.id,
          false,
        )
        .await?
        .community,
      )
    }
  }
}
