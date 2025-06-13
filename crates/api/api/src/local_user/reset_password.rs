use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::check_email_verified};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::{
  api::{PasswordReset, SuccessResponse},
  SiteView,
};
use lemmy_email::account::send_password_reset_email;
use lemmy_utils::error::LemmyResult;
use tracing::error;

pub async fn reset_password(
  data: Json<PasswordReset>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  let email = data.email.to_lowercase();
  // For security, errors are not returned.
  // https://github.com/LemmyNet/lemmy/issues/5277
  let _ = try_reset_password(&email, &context).await;
  Ok(Json(SuccessResponse::default()))
}

async fn try_reset_password(email: &str, context: &LemmyContext) -> LemmyResult<()> {
  let local_user_view = LocalUserView::find_by_email(&mut context.pool(), email).await?;
  let site_view = SiteView::read_local(&mut context.pool()).await?;

  check_email_verified(&local_user_view, &site_view)?;
  if let Err(e) =
    send_password_reset_email(&local_user_view, &mut context.pool(), context.settings()).await
  {
    error!("Failed to send password reset email: {}", e);
  }

  Ok(())
}
