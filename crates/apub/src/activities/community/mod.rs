use crate::{
  activities::send_lemmy_activity,
  activity_lists::AnnouncableActivities,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::activities::community::announce::AnnounceActivity,
};
use activitypub_federation::{config::Data, traits::Actor};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::person::PersonFollower;
use lemmy_utils::error::LemmyError;
use url::Url;
use tracing::warn;
use std::{env};

pub mod announce;
pub mod collection_add;
pub mod collection_remove;
pub mod lock_page;
pub mod report;
pub mod update;

/// This function sends all activities which are happening in a community to the right inboxes.
/// For example Create/Page, Add/Mod etc, but not private messages.
///
/// Activities are sent to the community itself if it lives on another instance. If the community
/// is local, the activity is directly wrapped into Announce and sent to community followers.
/// Activities are also sent to those who follow the actor (with exception of moderation activities).
///
/// * `activity` - The activity which is being sent
/// * `actor` - The user who is sending the activity
/// * `community` - Community inside which the activity is sent
/// * `inboxes` - Any additional inboxes the activity should be sent to (for example,
///               to the user who is being promoted to moderator)
/// * `is_mod_activity` - True for things like Add/Mod, these are not sent to user followers
pub(crate) async fn send_activity_in_community(
  activity: AnnouncableActivities,
  actor: &ApubPerson,
  community: &ApubCommunity,
  extra_inboxes: Vec<Url>,
  is_mod_action: bool,
  context: &Data<LemmyContext>,
) -> Result<(), LemmyError> {
  // send to any users which are mentioned or affected directly
  let mut inboxes = extra_inboxes;

  // send to user followers
  if !is_mod_action {
    inboxes.extend(
      &mut PersonFollower::list_followers(context.pool(), actor.id)
        .await?
        .into_iter()
        .map(|p| ApubPerson(p).shared_inbox_or_inbox()),
    );
  }

  if community.local {
    // send directly to community followers
    // PERFORMANCE CRISIS NOTE: if the activity is a Vote on a comment or post, this is the ideal point
    //    to match the activity type and SKIP this send call to not have to federate votes to all the instances
    //    following your community. Busy servers (examples: lemmy.world, lemm.ee, beehaw.org) have many instances
    //    following their local communities. This is the point where content is sent to each of those remote instances.
    //    Specifically targeting post votes and comment votes is a proposed emergency performance measure for overloaded senders.
    //    There are also PostgreSQL operations in this send activity that will also be bypassed, further reducing server overload.
    //    Server operators can set enviornment variable LEMMY_SKIP_FEDERATE_VOTES to trigger this code path.
    if env::var("LEMMY_SKIP_FEDERATE_VOTES").is_ok() {
      match activity {
          AnnouncableActivities::UndoVote(_)  |
          AnnouncableActivities::Vote(_) => {
            warn!("LEMMY_SKIP_FEDERATE_VOTES detected, SKIP outbound federation of Vote/UndoVote");
          },
          _ => {
            AnnounceActivity::send(activity.clone().try_into()?, community, context).await?;
          }
      }
    } else {
      AnnounceActivity::send(activity.clone().try_into()?, community, context).await?;
    }
  } else {
    // send to the community, which will then forward to followers
    // Another instance is home to the community, only one single outbound notificaiton is required to send a vote, so go ahead.
    inboxes.push(community.shared_inbox_or_inbox());
  }

  send_lemmy_activity(context, activity.clone(), actor, inboxes, false).await?;
  Ok(())
}
