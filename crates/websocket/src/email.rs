use crate::LemmyContext;
use lemmy_api_common::blocking;
use lemmy_db_schema::{
  newtypes::LocalUserId,
  source::email_verification::{EmailVerification, EmailVerificationForm},
  traits::Crud,
};
use lemmy_utils::{email::send_email, utils::generate_random_string, LemmyError};

pub async fn send_verification_email(
  local_user_id: LocalUserId,
  new_email: &str,
  username: &str,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  let settings = context.settings();
  let form = EmailVerificationForm {
    local_user_id,
    email: new_email.to_string(),
    verification_token: generate_random_string(),
  };
  // TODO: link should be replaced with a frontend route once that exists
  let verify_link = format!(
    "{}/api/v3/user/verify_email?token={}",
    settings.get_protocol_and_hostname(),
    &form.verification_token
  );
  blocking(context.pool(), move |conn| {
    EmailVerification::create(conn, &form)
  })
  .await??;

  let subject = format!("Verify your email address for {}", settings.hostname);
  let body = format!(
    concat!(
      "Please click the link below to verify your email address ",
      "for the account @{}@{}. Ignore this email if the account isn't yours.\n\n",
      "<a href=\"{}\"></a>"
    ),
    username, settings.hostname, verify_link
  );
  send_email(&subject, new_email, username, &body, &context.settings())?;

  Ok(())
}
