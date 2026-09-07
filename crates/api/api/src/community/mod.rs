use activitypub_federation::config::Data;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_deleted_removed,
};
use lemmy_db_schema::{
  source::community::{Community, CommunityActions, CommunityFollowerForm},
  traits::Followable,
};
use lemmy_db_schema_file::enums::{CommunityFollowerState, CommunityVisibility};
use lemmy_db_views_community_moderator::{CommunityModeratorView, CommunityPersonBanView};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub mod add_mod;
pub mod ban;
pub mod block;
pub mod follow;
pub mod multi_community_follow;
pub mod pending_follows;
pub mod random;
pub mod tag;
pub mod transfer;
pub mod update_notifications;

pub(super) async fn do_follow_community(
  community: Community,
  local_user_view: &LocalUserView,
  follow: bool,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let person_id = local_user_view.person.id;
  if follow {
    // Only run these checks for local community, in case of remote community the local
    // state may be outdated. Can't use check_community_user_action() here as it only allows
    // actions from existing followers for private community (so following would be impossible).
    if community.local {
      check_community_deleted_removed(&community)?;
      CommunityPersonBanView::check(&mut context.pool(), person_id, community.id).await?;
    }

    // Mods and local admins can already view private communities without approval, so we can also
    // approve their follow requests automatically.
    let is_mod = CommunityModeratorView::check_is_community_moderator(
      &mut context.pool(),
      community.id,
      person_id,
    )
    .await
    .is_ok();
    let is_local_admin =
      local_user_view.local_user.admin && local_user_view.person.local && community.local;
    let approval_required =
      community.visibility == CommunityVisibility::Private && !is_mod && !is_local_admin;

    let follow_state = if approval_required {
      // Private communities require manual approval
      CommunityFollowerState::ApprovalRequired
    } else if community.local {
      // Local follow is accepted immediately
      CommunityFollowerState::Accepted
    } else {
      // remote follow needs to be federated first
      CommunityFollowerState::Pending
    };
    let form = CommunityFollowerForm::new(community.id, person_id, follow_state);

    // Write to db
    CommunityActions::follow(&mut context.pool(), &form).await?;
  } else {
    CommunityActions::unfollow(&mut context.pool(), person_id, community.id).await?;
  }

  // Send the federated follow
  if !community.local {
    ActivityChannel::submit_activity(
      SendActivityData::FollowCommunity(community, local_user_view.person.clone(), follow),
      context,
    )?;
  }
  Ok(())
}
