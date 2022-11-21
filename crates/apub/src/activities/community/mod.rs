use crate::{
  activities::send_lemmy_activity,
  activity_lists::AnnouncableActivities,
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::community::announce::AnnounceActivity,
};
use activitypub_federation::{core::object_id::ObjectId, traits::Actor};
use lemmy_db_schema::source::person::PersonFollower;
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

pub mod add_mod;
pub mod announce;
pub mod remove_mod;
pub mod report;
pub mod update;

pub(crate) async fn send_activity_in_community(
  activity: AnnouncableActivities,
  actor: &ApubPerson,
  community: &ApubCommunity,
  mut inboxes: Vec<Url>,
  is_mod_action: bool,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  // send to user followers
  if !is_mod_action {
    let person_follower_inboxes = PersonFollower::list_followers(context.pool(), actor.id)
      .await?
      .into_iter()
      .map(|p| ApubPerson(p).shared_inbox_or_inbox())
      .collect();
    send_lemmy_activity(
      context,
      activity.clone(),
      actor,
      person_follower_inboxes,
      false,
    )
    .await?;
  }

  // send to community
  inboxes.push(community.shared_inbox_or_inbox());
  send_lemmy_activity(context, activity.clone(), actor, inboxes, false).await?;

  // send to community followers
  if community.local {
    AnnounceActivity::send(activity.try_into()?, community, context).await?;
  }

  Ok(())
}

#[tracing::instrument(skip_all)]
async fn get_community_from_moderators_url(
  moderators: &Url,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<ApubCommunity, LemmyError> {
  let community_id = Url::parse(&moderators.to_string().replace("/moderators", ""))?;
  ObjectId::new(community_id)
    .dereference(context, local_instance(context).await, request_counter)
    .await
}
