use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  site::{ApproveRegistrationApplication, RegistrationApplicationResponse},
  utils::{is_admin, local_user_view_from_jwt, send_application_approved_email},
};
use lemmy_db_schema::{
  source::{
    local_user::{LocalUser, LocalUserUpdateForm},
    registration_application::{RegistrationApplication, RegistrationApplicationUpdateForm},
  },
  traits::Crud,
  utils::diesel_option_overwrite,
};
use lemmy_db_views::structs::{LocalUserView, RegistrationApplicationView};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for ApproveRegistrationApplication {
  type Response = RegistrationApplicationResponse;

  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let data = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let app_id = data.id;

    // Only let admins do this
    is_admin(&local_user_view)?;

    // Update the registration with reason, admin_id
    let deny_reason = diesel_option_overwrite(data.deny_reason.clone());
    let app_form = RegistrationApplicationUpdateForm {
      admin_id: Some(Some(local_user_view.person.id)),
      deny_reason,
    };

    let registration_application =
      RegistrationApplication::update(&mut context.pool(), app_id, &app_form).await?;

    // Update the local_user row
    let local_user_form = LocalUserUpdateForm {
      accepted_application: Some(data.approve),
      ..Default::default()
    };

    let approved_user_id = registration_application.local_user_id;
    LocalUser::update(&mut context.pool(), approved_user_id, &local_user_form).await?;

    if data.approve {
      let approved_local_user_view =
        LocalUserView::read(&mut context.pool(), approved_user_id).await?;

      if approved_local_user_view.local_user.email.is_some() {
        send_application_approved_email(&approved_local_user_view, context.settings()).await?;
      }
    }

    // Read the view
    let registration_application =
      RegistrationApplicationView::read(&mut context.pool(), app_id).await?;

    Ok(Self::Response {
      registration_application,
    })
  }
}
