use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::ResendVerificationEmail,
  utils::send_verification_email_if_required,
  SuccessResponse,
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
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
    &context,
    &site_view.local_site,
    &local_user_view.local_user,
    &local_user_view.person,
  )
  .await?;

  Ok(Json(SuccessResponse::default()))
}
