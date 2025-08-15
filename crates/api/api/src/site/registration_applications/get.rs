use actix_web::web::{Data, Json, Path};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::newtypes::PersonId;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_registration_applications::{
  api::RegistrationApplicationResponse,
  RegistrationApplicationView,
};
use lemmy_utils::error::LemmyResult;

/// Lists registration applications, filterable by undenied only.
pub async fn get_registration_application(
  person_id: Path<PersonId>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<RegistrationApplicationResponse>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  // Read the view
  let registration_application =
    RegistrationApplicationView::read_by_person(&mut context.pool(), person_id.into_inner())
      .await?;

  Ok(Json(RegistrationApplicationResponse {
    registration_application,
  }))
}
