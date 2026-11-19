use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_banned_or_deleted};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::{
  SiteView,
  api::{ResendVerificationEmail, SuccessResponse},
};
use lemmy_email::account::resend_verification_email as send_limited;
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
  {
    return Ok(());
  }

  // Per-account limit is enforced in the database, so it is shared by all backend processes,
  // survives restarts and always uses the current admin setting.
  send_limited(
    &site_view.local_site,
    &site_view.local_site_rate_limit,
    &local_user_view,
    &mut context.pool(),
    context.settings(),
  )
  .await?;

  Ok(())
}
