use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  site::{GetRegistrationApplication, RegistrationApplicationResponse},
  utils::is_admin,
};
use lemmy_db_views::structs::{LocalUserView, RegistrationApplicationView};
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

/// Lists registration applications, filterable by undenied only.
pub async fn get_registration_application(
  data: Query<GetRegistrationApplication>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<RegistrationApplicationResponse>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  // Read the view
  let registration_application =
    RegistrationApplicationView::read_by_person(&mut context.pool(), data.person_id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindRegistrationApplication)?;

  Ok(Json(RegistrationApplicationResponse {
    registration_application,
  }))
}
