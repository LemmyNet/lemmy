use crate::{
  activities::send_lemmy_activity,
  activity_lists::AnnouncableActivities,
  protocol::activities::community::announce::AnnounceActivity,
};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId, traits::Actor};
use lemmy_api_common::context::LemmyContext;
use lemmy_apub_objects::objects::{
  community::ApubCommunity,
  instance::ApubSite,
  person::ApubPerson,
  PostOrComment,
};
use lemmy_db_schema::{
  source::{
    activity::ActivitySendTargets,
    person::{Person, PersonActions},
    site::Site,
  },
  traits::Crud,
};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_utils::error::LemmyResult;

pub mod announce;
pub mod collection_add;
pub mod collection_remove;
pub mod lock_page;
pub mod report;
pub mod resolve_report;
pub mod update;

/// This function sends all activities which are happening in a community to the right inboxes.
/// For example Create/Page, Add/Mod etc, but not private messages.
///
/// Activities are sent to the community itself if it lives on another instance. If the community
/// is local, the activity is directly wrapped into Announce and sent to community followers.
/// Activities are also sent to those who follow the actor (with exception of moderation
/// activities).
///
/// * `activity` - The activity which is being sent
/// * `actor` - The user who is sending the activity
/// * `community` - Community inside which the activity is sent
/// * `inboxes` - Any additional inboxes the activity should be sent to (for example, to the user
///   who is being promoted to moderator)
/// * `is_mod_activity` - True for things like Add/Mod, these are not sent to user followers
pub(crate) async fn send_activity_in_community(
  activity: AnnouncableActivities,
  actor: &ApubPerson,
  community: &ApubCommunity,
  extra_inboxes: ActivitySendTargets,
  is_mod_action: bool,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  // If community is local only, don't send anything out
  if !community.visibility.can_federate() {
    return Ok(());
  }

  // send to any users which are mentioned or affected directly
  let mut inboxes = extra_inboxes;

  // send to user followers
  if !is_mod_action {
    inboxes.add_inboxes(
      PersonActions::list_followers(&mut context.pool(), actor.id)
        .await?
        .into_iter()
        .map(|p| ApubPerson(p).shared_inbox_or_inbox()),
    );
  }

  if community.local {
    // send directly to community followers
    AnnounceActivity::send(activity.clone().try_into()?, community, context).await?;
  } else {
    // send to the community, which will then forward to followers
    inboxes.add_inbox(community.shared_inbox_or_inbox());
  }

  send_lemmy_activity(context, activity.clone(), actor, inboxes, false).await?;
  Ok(())
}

async fn report_inboxes(
  object_id: ObjectId<PostOrComment>,
  community: &ApubCommunity,
  context: &Data<LemmyContext>,
) -> LemmyResult<ActivitySendTargets> {
  // send report to the community where object was posted
  let mut inboxes = ActivitySendTargets::to_inbox(community.shared_inbox_or_inbox());

  if community.local {
    // send to all moderators
    let moderators =
      CommunityModeratorView::for_community(&mut context.pool(), community.id).await?;
    for m in moderators {
      inboxes.add_inbox(m.moderator.inbox_url.into());
    }

    // also send report to user's home instance if possible
    let object_creator_id = match object_id.dereference_local(context).await? {
      PostOrComment::Left(p) => p.creator_id,
      PostOrComment::Right(c) => c.creator_id,
    };
    let object_creator = Person::read(&mut context.pool(), object_creator_id).await?;
    let object_creator_site: Option<ApubSite> =
      Site::read_from_instance_id(&mut context.pool(), object_creator.instance_id)
        .await
        .ok()
        .map(Into::into);
    if let Some(inbox) = object_creator_site.map(|s| s.shared_inbox_or_inbox()) {
      inboxes.add_inbox(inbox);
    }
  }
  Ok(inboxes)
}
