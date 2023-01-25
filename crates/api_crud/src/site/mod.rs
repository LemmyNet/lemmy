use lemmy_db_schema::source::local_site::RegistrationMode;
use lemmy_utils::error::LemmyError;

mod create;
mod read;
mod update;

pub fn check_application_question(
  application_question: &Option<Option<String>>,
  registration_mode: RegistrationMode,
) -> Result<(), LemmyError> {
  if registration_mode == RegistrationMode::RequireApplication
    && application_question.as_ref().unwrap_or(&None).is_none()
  {
    Err(LemmyError::from_message("application_question_required"))
  } else {
    Ok(())
  }
}
