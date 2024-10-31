use lemmy_api_common::utils::is_admin;
use lemmy_db_schema::CommunityVisibility;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub mod create;
pub mod delete;
pub mod list;
pub mod remove;
pub mod update;

/// For now only admins can make communities private, in order to prevent abuse.
/// Need to implement admin approval for new communities to get rid of this.
fn check_community_visibility_allowed(
  visibility: Option<CommunityVisibility>,
  local_user_view: &LocalUserView,
) -> LemmyResult<()> {
  if visibility == Some(lemmy_db_schema::CommunityVisibility::Private) {
    is_admin(local_user_view)?;
  }
  Ok(())
}
