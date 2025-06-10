use actix_web::web::{Data, Json};
use chrono::Utc;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::local_user::{LocalUser, LocalUserUpdateForm};
use lemmy_db_views_api_misc::SuccessResponse;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn donation_dialog_shown(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let form = LocalUserUpdateForm {
    last_donation_notification_at: Some(Utc::now()),
    ..Default::default()
  };
  LocalUser::update(&mut context.pool(), local_user_view.local_user.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}
