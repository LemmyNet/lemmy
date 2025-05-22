use lemmy_api_common::{context::LemmyContext, LemmyErrorType};
use lemmy_db_schema::{
  newtypes::MultiCommunityId,
  source::multi_community::MultiCommunity,
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub mod create;
pub mod create_entry;
pub mod delete_entry;
pub mod get;
pub mod list;
pub mod update;

/// Check that current user is creator of multi-comm and can modify it.
async fn check_multi_community_creator(
  id: MultiCommunityId,
  local_user_view: &LocalUserView,
  context: &LemmyContext,
) -> LemmyResult<()> {
  let read = MultiCommunity::read(&mut context.pool(), id).await?;
  if read.creator_id != local_user_view.person.id {
    return Err(LemmyErrorType::MultiCommunityUpdateWrongUser.into());
  }
  Ok(())
}
