use crate::{send::send_email, user_email, user_language};
use lemmy_db_schema::{
  sensitive::SensitiveString,
  source::{
    email_verification::{EmailVerification, EmailVerificationForm},
    local_site::LocalSite,
    password_reset_request::PasswordResetRequest,
  },
  utils::DbPool,
};
use lemmy_db_schema_file::enums::RegistrationMode;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{
  error::LemmyResult,
  settings::structs::Settings,
  utils::markdown::markdown_to_html,
};

pub async fn send_password_reset_email(
  user: &LocalUserView,
  pool: &mut DbPool<'_>,
  settings: &'static Settings,
) -> LemmyResult<()> {
  // Generate a random token
  let token = uuid::Uuid::new_v4().to_string();

  let lang = user_language(user);
  let subject = lang.password_reset_subject(&user.person.name);
  let protocol_and_hostname = settings.get_protocol_and_hostname();
  let reset_link = format!("{}/password_change/{}", protocol_and_hostname, &token);
  let email = user_email(user)?;
  let body = lang.password_reset_body(reset_link, &user.person.name);
  send_email(subject, email, user.person.name.clone(), body, settings);

  // Insert the row after successful send, to avoid using daily reset limit while
  // email sending is broken.
  let local_user_id = user.local_user.id;
  PasswordResetRequest::create(pool, local_user_id, token.clone()).await?;
  Ok(())
}

/// Send a verification email
pub async fn send_verification_email(
  local_site: &LocalSite,
  user: &LocalUserView,
  new_email: SensitiveString,
  pool: &mut DbPool<'_>,
  settings: &'static Settings,
) -> LemmyResult<()> {
  let form = EmailVerificationForm {
    local_user_id: user.local_user.id,
    email: new_email.to_string(),
    verification_token: uuid::Uuid::new_v4().to_string(),
  };
  let verify_link = format!(
    "{}/verify_email/{}",
    settings.get_protocol_and_hostname(),
    &form.verification_token
  );
  EmailVerification::create(pool, &form).await?;

  let lang = user_language(user);
  let subject = lang.verify_email_subject(&settings.hostname);

  // If an application is required, use a translation that includes that warning.
  let body = if local_site.registration_mode == RegistrationMode::RequireApplication {
    lang.verify_email_body_with_application(&settings.hostname, &user.person.name, verify_link)
  } else {
    lang.verify_email_body(&settings.hostname, &user.person.name, verify_link)
  };

  send_email(subject, new_email, user.person.name.clone(), body, settings);
  Ok(())
}

/// Returns true if email was sent.
pub async fn send_verification_email_if_required(
  local_site: &LocalSite,
  user: &LocalUserView,
  pool: &mut DbPool<'_>,
  settings: &'static Settings,
) -> LemmyResult<bool> {
  if !user.local_user.admin
    && local_site.require_email_verification
    && !user.local_user.email_verified
  {
    let email = user_email(user)?;
    send_verification_email(local_site, user, email, pool, settings).await?;
    Ok(true)
  } else {
    Ok(false)
  }
}

pub fn send_application_approved_email(
  user: &LocalUserView,
  settings: &'static Settings,
) -> LemmyResult<()> {
  let lang = user_language(user);
  let subject = lang.registration_approved_subject(&user.person.name);
  let email = user_email(user)?;
  let body = lang.registration_approved_body(&settings.hostname);
  send_email(subject, email, user.person.name.clone(), body, settings);
  Ok(())
}

pub fn send_application_denied_email(
  user: &LocalUserView,
  deny_reason: Option<String>,
  settings: &'static Settings,
) -> LemmyResult<()> {
  let lang = user_language(user);
  let subject = lang.registration_denied_subject(&user.person.name);
  let email = user_email(user)?;
  let body = match deny_reason {
    Some(deny_reason) => {
      let markdown = markdown_to_html(&deny_reason);
      lang.registration_denied_reason_body(&settings.hostname, &markdown)
    }
    None => lang.registration_denied_body(&settings.hostname),
  };
  send_email(subject, email, user.person.name.clone(), body, settings);
  Ok(())
}

pub fn send_email_verified_email(
  user: &LocalUserView,
  settings: &'static Settings,
) -> LemmyResult<()> {
  let lang = user_language(user);
  let subject = lang.email_verified_subject(&user.person.name);
  let email = user_email(user)?;
  let body = lang.email_verified_body();
  send_email(
    subject,
    email,
    user.person.name.clone(),
    body.to_string(),
    settings,
  );
  Ok(())
}
