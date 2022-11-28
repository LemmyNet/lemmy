use actix_web::web::Data;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_utils::{error::LemmyError, ConnectionId};

mod comment;
mod community;
mod custom_emoji;
mod post;
mod private_message;
mod site;
mod user;

#[async_trait::async_trait(?Send)]
pub trait PerformCrud {
  type Response: serde::ser::Serialize + Send;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError>;
}

/// Make sure if applications are required, that there is an application questionnaire
pub fn check_application_question(
  application_question: &Option<Option<String>>,
  local_site: &LocalSite,
  require_application: &Option<bool>,
) -> Result<(), LemmyError> {
  if require_application.unwrap_or(false)
    && (application_question == &Some(None)
      || (application_question.is_none() && local_site.application_question.is_none()))
  {
    return Err(LemmyError::from_message("application_question_required"));
  }
  Ok(())
}
