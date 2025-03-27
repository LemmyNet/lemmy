use crate::{send_email, send_email_to_user, user_language};
use lemmy_db_schema::{
  source::{
    email_verification::{EmailVerification, EmailVerificationForm},
    local_site::LocalSite,
    local_user::LocalUser,
    password_reset_request::PasswordResetRequest,
    person::Person,
  },
  utils::DbPool,
  RegistrationMode,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};

pub async fn send_password_reset_email(
  user: &LocalUserView,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> LemmyResult<()> {
  // Generate a random token
  let token = uuid::Uuid::new_v4().to_string();

  let lang = user_language(&user.local_user);
  let subject = &lang.password_reset_subject(&user.person.name);
  let protocol_and_hostname = settings.get_protocol_and_hostname();
  let reset_link = format!("{}/password_change/{}", protocol_and_hostname, &token);
  let body = &lang.password_reset_body(reset_link, &user.person.name);
  send_email_to_user(user, subject, body, settings).await;

  // Insert the row after successful send, to avoid using daily reset limit while
  // email sending is broken.
  let local_user_id = user.local_user.id;
  PasswordResetRequest::create(pool, local_user_id, token.clone()).await?;
  Ok(())
}

/// Send a verification email
pub async fn send_verification_email(
  local_site: &LocalSite,
  local_user: &LocalUser,
  person: &Person,
  new_email: &str,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> LemmyResult<()> {
  let form = EmailVerificationForm {
    local_user_id: local_user.id,
    email: new_email.to_string(),
    verification_token: uuid::Uuid::new_v4().to_string(),
  };
  let verify_link = format!(
    "{}/verify_email/{}",
    settings.get_protocol_and_hostname(),
    &form.verification_token
  );
  EmailVerification::create(pool, &form).await?;

  let lang = user_language(local_user);
  let subject = lang.verify_email_subject(&settings.hostname);

  // If an application is required, use a translation that includes that warning.
  let body = if local_site.registration_mode == RegistrationMode::RequireApplication {
    lang.verify_email_body_with_application(&settings.hostname, &person.name, verify_link)
  } else {
    lang.verify_email_body(&settings.hostname, &person.name, verify_link)
  };

  send_email(&subject, new_email, &person.name, &body, settings).await
}

/// Returns true if email was sent.
pub async fn send_verification_email_if_required(
  local_site: &LocalSite,
  local_user: &LocalUser,
  person: &Person,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> LemmyResult<bool> {
  let email = &local_user
    .email
    .clone()
    .ok_or(LemmyErrorType::EmailRequired)?;

  if !local_user.admin && local_site.require_email_verification && !local_user.email_verified {
    send_verification_email(local_site, local_user, person, email, pool, settings).await?;
    Ok(true)
  } else {
    Ok(false)
  }
}

pub async fn send_application_approved_email(
  user: &LocalUserView,
  settings: &Settings,
) -> LemmyResult<()> {
  let lang = user_language(&user.local_user);
  let subject = lang.registration_approved_subject(&user.person.name);
  let body = lang.registration_approved_body(&settings.hostname);
  send_email_to_user(user, &subject, &body, settings).await;
  Ok(())
}

pub async fn send_application_denied_email(
  user: &LocalUserView,
  deny_reason: Option<String>,
  settings: &Settings,
) -> LemmyResult<()> {
  let lang = user_language(&user.local_user);
  let subject = lang.registration_denied_subject(&user.person.name);
  let body = lang.new_registration_denied_body(
    &settings.hostname,
    deny_reason.unwrap_or("unknown".to_string()),
  );
  send_email_to_user(user, &subject, &body, settings).await;
  Ok(())
}

pub async fn send_email_verified_email(local_user_view: &LocalUserView, settings: &Settings) {
  let lang = user_language(&local_user_view.local_user);
  let subject = lang.email_verified_subject(&local_user_view.person.name);
  let body = lang.email_verified_body();
  send_email_to_user(local_user_view, &subject, body, settings).await;
}
