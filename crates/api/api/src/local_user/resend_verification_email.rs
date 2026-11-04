use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_banned_or_deleted};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::{
  SiteView,
  api::{ResendVerificationEmail, SuccessResponse},
};
use lemmy_email::account::send_verification_email_if_required;
use lemmy_utils::error::LemmyResult;

pub async fn resend_verification_email(
  Json(data): Json<ResendVerificationEmail>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  let email = data.email.trim().to_lowercase();

  // For security, errors are not returned.
  // https://github.com/LemmyNet/lemmy/issues/5277
  let _ = try_resend_verification_email(&email, &context).await;

  Ok(Json(SuccessResponse::default()))
}

async fn try_resend_verification_email(email: &str, context: &LemmyContext) -> LemmyResult<()> {
  // Fetch that email
  let local_user_view = LocalUserView::find_by_email(&mut context.pool(), email).await?;
  check_local_user_banned_or_deleted(&local_user_view)?;

  let site_view = SiteView::read_local(&mut context.pool()).await?;

  if !site_view.local_site.email_verification_required
    || local_user_view.local_user.admin
    || local_user_view.local_user.email_verified
    || !context
      .rate_limit_cell()
      .allow_verification_email(local_user_view.local_user.id.0)
      .await
  {
    return Ok(());
  }

  send_verification_email_if_required(
    &site_view.local_site,
    &local_user_view,
    &mut context.pool(),
    context.settings(),
  )
  .await?;

  Ok(())
}
