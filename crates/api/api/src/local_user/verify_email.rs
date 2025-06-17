use actix_web::web::{Data, Json};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::{
  email_verification::EmailVerification,
  local_user::{LocalUser, LocalUserUpdateForm},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::{
  api::{SuccessResponse, VerifyEmail},
  SiteView,
};
use lemmy_email::{account::send_email_verified_email, admin::send_new_applicant_email_to_admins};
use lemmy_utils::error::LemmyResult;

pub async fn verify_email(
  data: Json<VerifyEmail>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let token = data.token.clone();
  let verification = EmailVerification::read_for_token(&mut context.pool(), &token).await?;
  let local_user_id = verification.local_user_id;
  let local_user_view = LocalUserView::read(&mut context.pool(), local_user_id).await?;

  // Check if their email has already been verified once, before this
  let email_already_verified = local_user_view.local_user.email_verified;

  let form = LocalUserUpdateForm {
    // necessary in case this is a new signup
    email_verified: Some(true),
    // necessary in case email of an existing user was changed
    email: Some(Some(verification.email)),
    ..Default::default()
  };

  LocalUser::update(&mut context.pool(), local_user_id, &form).await?;

  EmailVerification::delete_old_tokens_for_local_user(&mut context.pool(), local_user_id).await?;

  // Send out notification about registration application to admins if enabled, and the user hasn't
  // already been verified.
  if site_view.local_site.application_email_admins && !email_already_verified {
    send_new_applicant_email_to_admins(
      &local_user_view.person.name,
      &mut context.pool(),
      context.settings(),
    )
    .await?;
  }

  send_email_verified_email(&local_user_view, context.settings()).await?;

  Ok(Json(SuccessResponse::default()))
}
