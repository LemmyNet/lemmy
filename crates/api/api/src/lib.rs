use lemmy_api_utils::{context::LemmyContext, utils::is_mod_or_admin_opt};
use lemmy_db_schema_file::newtypes::CommunityId;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  utils::slurs::check_slurs,
};
use regex::Regex;

pub mod comment;
pub mod community;
pub mod federation;
pub mod local_user;
pub mod post;
pub mod reports;
pub mod site;
pub mod sitemap;

/// Check size of report
pub(crate) fn check_report_reason(reason: &str, slur_regex: &Regex) -> LemmyResult<()> {
  check_slurs(reason, slur_regex)?;
  if reason.is_empty() {
    Err(LemmyErrorType::ReportReasonRequired.into())
  } else if reason.chars().count() > 1000 {
    Err(LemmyErrorType::ReportTooLong.into())
  } else {
    Ok(())
  }
}
/// Only show the modlog names if:
/// You're an admin or
/// You're fetching the modlog for a single community, and you're a mod
/// (Alternatively !admin/mod)
async fn hide_modlog_names(
  local_user_view: Option<&LocalUserView>,
  community_id: Option<CommunityId>,
  context: &LemmyContext,
) -> bool {
  if let Some(community_id) = community_id {
    is_mod_or_admin_opt(&mut context.pool(), local_user_view, Some(community_id))
      .await
      .is_err()
  } else {
    !local_user_view
      .map(|l| l.local_user.admin)
      .unwrap_or_default()
  }
}
