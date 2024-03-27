use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

pub mod add_admin;
pub mod ban_person;
pub mod block;
pub mod change_password;
pub mod change_password_after_reset;
pub mod generate_totp_secret;
pub mod get_captcha;
pub mod list_banned;
pub mod list_logins;
pub mod list_media;
pub mod login;
pub mod logout;
pub mod notifications;
pub mod report_count;
pub mod reset_password;
pub mod save_settings;
pub mod update_totp;
pub mod validate_auth;
pub mod verify_email;

/// Check if the user's email is verified if email verification is turned on
/// However, skip checking verification if the user is an admin
fn check_email_verified(local_user_view: &LocalUserView, site_view: &SiteView) -> LemmyResult<()> {
  if !local_user_view.local_user.admin
    && site_view.local_site.require_email_verification
    && !local_user_view.local_user.email_verified
  {
    Err(LemmyErrorType::EmailNotVerified)?
  }
  Ok(())
}
