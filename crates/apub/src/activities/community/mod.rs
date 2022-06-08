use crate::{
  activities::send_lemmy_activity,
  activity_lists::AnnouncableActivities,
  local_instance,
  objects::community::ApubCommunity,
  protocol::activities::community::announce::AnnounceActivity,
  ActorType,
};
use activitypub_federation::{core::object_id::ObjectId, traits::Actor};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

pub mod add_mod;
pub mod announce;
pub mod remove_mod;
pub mod report;
pub mod update;

#[tracing::instrument(skip_all)]
pub(crate) async fn send_activity_in_community<ActorT>(
  activity: AnnouncableActivities,
  actor: &ActorT,
  community: &ApubCommunity,
  mut inboxes: Vec<Url>,
  context: &LemmyContext,
) -> Result<(), LemmyError>
where
  ActorT: Actor + ActorType,
{
  inboxes.push(community.shared_inbox_or_inbox());
  send_lemmy_activity(context, activity.clone(), actor, inboxes, false).await?;

  if community.local {
    AnnounceActivity::send(activity, community, context).await?;
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
    .dereference(context, local_instance(context), request_counter)
    .await
}
