use crate::local_user::check_email_verified;
use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::PasswordReset,
  utils::send_password_reset_email,
  SuccessResponse,
};
use lemmy_db_schema::source::password_reset_request::PasswordResetRequest;
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn reset_password(
  data: Json<PasswordReset>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  // Fetch that email
  let email = data.email.to_lowercase();
  let local_user_view = LocalUserView::find_by_email(&mut context.pool(), &email)
    .await?
    .ok_or(LemmyErrorType::IncorrectLogin)?;

  // Check for too many attempts (to limit potential abuse)
  let recent_resets_count = PasswordResetRequest::get_recent_password_resets_count(
    &mut context.pool(),
    local_user_view.local_user.id,
  )
  .await?;
  if recent_resets_count >= 3 {
    Err(LemmyErrorType::PasswordResetLimitReached)?
  }
  let site_view = SiteView::read_local(&mut context.pool())
    .await?
    .ok_or(LemmyErrorType::LocalSiteNotSetup)?;
  check_email_verified(&local_user_view, &site_view)?;

  // Email the pure token to the user.
  send_password_reset_email(&local_user_view, &mut context.pool(), context.settings()).await?;
  Ok(Json(SuccessResponse::default()))
}
