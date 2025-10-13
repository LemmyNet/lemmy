use activitypub_federation::config::Data;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
};
use lemmy_db_schema::source::{multi_community::MultiCommunity, person::Person};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub mod create;
pub mod create_entry;
pub mod delete_entry;
pub mod list;
pub mod update;

/// Check that current user is creator of multi-comm and can modify it.
fn check_multi_community_creator(
  multi: &MultiCommunity,
  local_user_view: &LocalUserView,
) -> LemmyResult<()> {
  if multi.local && local_user_view.local_user.admin {
    Ok(())
  } else if multi.creator_id != local_user_view.person.id {
    Err(LemmyErrorType::MultiCommunityUpdateWrongUser.into())
  } else {
    Ok(())
  }
}

fn send_federation_update(
  multi: MultiCommunity,
  person: Person,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  ActivityChannel::submit_activity(
    SendActivityData::UpdateMultiCommunity(multi, person),
    context,
  )
}
