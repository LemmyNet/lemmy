use actix_web::web::{Data, Json};
use chrono::Utc;
use lemmy_api_common::{context::LemmyContext, SuccessResponse};
use lemmy_db_schema::source::local_user::{LocalUser, LocalUserUpdateForm};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn donation_dialog_shown(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let form = LocalUserUpdateForm {
    last_donation_notification: Some(Utc::now()),
    ..Default::default()
  };
  LocalUser::update(&mut context.pool(), local_user_view.local_user.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}
