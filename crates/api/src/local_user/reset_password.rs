use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::PasswordReset,
  utils::send_password_reset_email,
  SuccessResponse,
};
use lemmy_db_schema::source::password_reset_request::PasswordResetRequest;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn reset_password(
  data: Json<PasswordReset>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  // Fetch that email
  let email = data.email.to_lowercase();
  let local_user_view = LocalUserView::find_by_email(&mut context.pool(), &email)
    .await
    .with_lemmy_type(LemmyErrorType::IncorrectLogin)?;

  // Check for too many attempts (to limit potential abuse)
  let recent_resets_count = PasswordResetRequest::get_recent_password_resets_count(
    &mut context.pool(),
    local_user_view.local_user.id,
  )
  .await?;
  if recent_resets_count >= 3 {
    Err(LemmyErrorType::PasswordResetLimitReached)?
  }

  // Email the pure token to the user.
  send_password_reset_email(&local_user_view, &mut context.pool(), context.settings()).await?;
  Ok(Json(SuccessResponse::default()))
}
