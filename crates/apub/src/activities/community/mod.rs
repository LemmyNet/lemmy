use crate::{
  activities::community::announce::{AnnouncableActivities, AnnounceActivity},
  check_is_apub_id_valid,
  insert_activity,
  objects::community::ApubCommunity,
  send_lemmy_activity,
  CommunityType,
};
use itertools::Itertools;
use lemmy_apub_lib::traits::ActorType;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

pub mod add_mod;
pub mod announce;
pub mod block_user;
pub mod remove_mod;
pub mod undo_block_user;
pub mod update;

async fn list_community_follower_inboxes(
  community: &ApubCommunity,
  additional_inboxes: Vec<Url>,
  context: &LemmyContext,
) -> Result<Vec<Url>, LemmyError> {
  Ok(
    vec![
      community
        .get_follower_inboxes(context.pool(), &context.settings())
        .await?,
      additional_inboxes,
    ]
    .iter()
    .flatten()
    .unique()
    .filter(|inbox| inbox.host_str() != Some(&context.settings().hostname))
    .filter(|inbox| check_is_apub_id_valid(inbox, false, &context.settings()).is_ok())
    .map(|inbox| inbox.to_owned())
    .collect(),
  )
}

pub(crate) async fn send_to_community<T: ActorType>(
  activity: AnnouncableActivities,
  activity_id: &Url,
  actor: &T,
  community: &ApubCommunity,
  additional_inboxes: Vec<Url>,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  // if this is a local community, we need to do an announce from the community instead
  if community.local {
    insert_activity(activity_id, activity.clone(), true, false, context.pool()).await?;
    AnnounceActivity::send(activity, community, additional_inboxes, context).await?;
  } else {
    let mut inboxes = additional_inboxes;
    inboxes.push(community.shared_inbox_or_inbox_url());
    send_lemmy_activity(context, &activity, activity_id, actor, inboxes, false).await?;
  }

  Ok(())
}
