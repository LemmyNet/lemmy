use actix_web::web::{Data, Json};
use chrono::{DateTime, Utc};
use lemmy_api_common::{context::LemmyContext, person::DonationDialogShown, SuccessResponse};
use lemmy_db_schema::source::local_user::{LocalUser, LocalUserUpdateForm};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn donation_dialog_shown(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
  data: Json<DonationDialogShown>,
) -> LemmyResult<Json<SuccessResponse>> {
  let last = if data.hide_permanently.unwrap_or_default() {
    DateTime::<Utc>::MAX_UTC
  } else {
    Utc::now()
  };
  let form = LocalUserUpdateForm {
    last_donation_notification: Some(last),
    ..Default::default()
  };
  LocalUser::update(&mut context.pool(), local_user_view.local_user.id, &form).await?;

  Ok(Json(SuccessResponse::default()))
}
