use actix_web::web::{Data, Json};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::{
  api::{ResendVerificationEmail, SuccessResponse},
  SiteView,
};
use lemmy_email::account::send_verification_email_if_required;
use lemmy_utils::error::LemmyResult;

pub async fn resend_verification_email(
  data: Json<ResendVerificationEmail>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let email = data.email.to_string();

  // Fetch that email
  let local_user_view = LocalUserView::find_by_email(&mut context.pool(), &email).await?;

  send_verification_email_if_required(
    &site_view.local_site,
    &local_user_view,
    &mut context.pool(),
    context.settings(),
  )
  .await?;

  Ok(Json(SuccessResponse::default()))
}
