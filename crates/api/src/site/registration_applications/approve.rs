use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  site::{ApproveRegistrationApplication, RegistrationApplicationResponse},
  utils::{
    is_admin,
    local_user_view_from_jwt,
    send_application_approved_email,
    send_application_denied_email,
  },
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
    let deny_reason = diesel_option_overwrite(&data.deny_reason);
    let app_form = RegistrationApplicationUpdateForm {
      admin_id: Some(Some(local_user_view.person.id)),
      deny_reason,
    };

    let registration_application =
      RegistrationApplication::update(context.pool(), app_id, &app_form).await?;

    // Update the local_user row
    let local_user_form = LocalUserUpdateForm::builder()
      .accepted_application(Some(data.approve))
      .build();

    let applicant_user_id = registration_application.local_user_id;
    LocalUser::update(context.pool(), applicant_user_id, &local_user_form).await?;

    // Handle approval/denial
    let applicant_local_user_view = LocalUserView::read(context.pool(), applicant_user_id).await?;
    if applicant_local_user_view.local_user.email.is_some() {
      if data.approve {
        // Approval
        send_application_approved_email(&applicant_local_user_view, context.settings())?;
      } else if !data.approve {
        // Rejection
        let deny_msg = &data.deny_reason.clone().unwrap_or_default();
        send_application_denied_email(&applicant_local_user_view, context.settings(), deny_msg)
          .await?;
      }
    }

    // Read the view
    let registration_application =
      RegistrationApplicationView::read(context.pool(), app_id).await?;

    Ok(Self::Response {
      registration_application,
    })
  }
}
