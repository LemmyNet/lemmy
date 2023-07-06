use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  site::{GetRegistrationRequirements, GetRegistrationRequirementsResponse},
};
use lemmy_db_schema::{source::local_site::LocalSite, RegistrationMode};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for GetRegistrationRequirements {
  type Response = GetRegistrationRequirementsResponse;

  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let local_site = LocalSite::read(context.pool()).await?;

    let answer_required = local_site.registration_mode == RegistrationMode::RequireApplication;

    Ok(Self::Response {
      question: local_site.application_question,
      answer_required,
      captcha_required: local_site.captcha_enabled,
      email_verification_required: local_site.require_email_verification,
    })
  }
}
