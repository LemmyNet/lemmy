use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  site::{ApproveRegistrationApplication, RegistrationApplicationResponse},
  utils::{is_admin, send_application_approved_email},
};
use lemmy_db_schema::{
  source::{
    local_user::{LocalUser, LocalUserUpdateForm},
    registration_application::{RegistrationApplication, RegistrationApplicationUpdateForm},
  },
  traits::Crud,
  utils::{diesel_string_update, get_conn},
};
use lemmy_db_views::structs::{LocalUserView, RegistrationApplicationView};
use lemmy_utils::{
  error::{LemmyError, LemmyResult},
  LemmyErrorType,
};

pub async fn approve_registration_application(
  data: Json<ApproveRegistrationApplication>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<RegistrationApplicationResponse>> {
  let app_id = data.id;

  // Only let admins do this
  is_admin(&local_user_view)?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let tx_data = data.clone();
  let approved_user_id = conn
    .build_transaction()
    .run(|conn| {
      Box::pin(async move {
        // Update the registration with reason, admin_id
        let deny_reason = diesel_string_update(tx_data.deny_reason.as_deref());
        let app_form = RegistrationApplicationUpdateForm {
          admin_id: Some(Some(local_user_view.person.id)),
          deny_reason,
        };

        let registration_application =
          RegistrationApplication::update(&mut conn.into(), app_id, &app_form).await?;

        // Update the local_user row
        let local_user_form = LocalUserUpdateForm {
          accepted_application: Some(tx_data.approve),
          ..Default::default()
        };

        let approved_user_id = registration_application.local_user_id;
        LocalUser::update(&mut conn.into(), approved_user_id, &local_user_form).await?;

        Ok::<_, LemmyError>(approved_user_id)
      }) as _
    })
    .await?;

  if data.approve {
    let approved_local_user_view = LocalUserView::read(&mut context.pool(), approved_user_id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindLocalUser)?;
    if approved_local_user_view.local_user.email.is_some() {
      // Email sending may fail, but this won't revert the application approval
      send_application_approved_email(&approved_local_user_view, context.settings()).await?;
    }
  };

  // Read the view
  let registration_application = RegistrationApplicationView::read(&mut context.pool(), app_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindRegistrationApplication)?;

  Ok(Json(RegistrationApplicationResponse {
    registration_application,
  }))
}
