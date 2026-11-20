use crate::{send::send_email, user_email, user_language};
use lemmy_db_schema::source::{
  email_verification::{EmailVerification, EmailVerificationForm},
  local_site::LocalSite,
  local_site_rate_limit::LocalSiteRateLimit,
  password_reset_request::PasswordResetRequest,
};
use lemmy_db_schema_file::enums::RegistrationMode;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{connection::DbPool, sensitive::SensitiveString};
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

  let lang = user_language(&user.local_user);
  let subject = lang.password_reset_subject(&user.person.name);
  let protocol_and_hostname = settings.get_protocol_and_hostname();
  let reset_link = format!("{}/password_change/{}", protocol_and_hostname, token);
  let email = user_email(user)?;
  let body = lang.password_reset_body(reset_link, &user.person.name);
  send_email(subject, email, user.person.name.clone(), body, settings);

  // Store the token. Password resets have no per-account limit, only the per-IP register rate
  // limit applies.
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
  let form = verification_form(user, &new_email);
  EmailVerification::create(pool, &form).await?;
  send_verification_link(local_site, user, new_email, &form, settings);
  Ok(())
}

/// Resend the verification email, unless the account already received
/// `max_requests` verification emails within `interval_seconds`. Returns true if sent.
pub async fn resend_verification_email(
  local_site: &LocalSite,
  rate_limit: &LocalSiteRateLimit,
  user: &LocalUserView,
  pool: &mut DbPool<'_>,
  settings: &'static Settings,
) -> LemmyResult<bool> {
  let email = user_email(user)?;
  let form = verification_form(user, &email);
  let created = EmailVerification::create_with_account_limit(
    pool,
    &form,
    rate_limit.resend_verification_email_per_account_max_requests,
    rate_limit.resend_verification_email_per_account_interval_seconds,
  )
  .await?;
  if created.is_none() {
    return Ok(false);
  }
  send_verification_link(local_site, user, email, &form, settings);
  Ok(true)
}

fn verification_form(user: &LocalUserView, email: &SensitiveString) -> EmailVerificationForm {
  EmailVerificationForm {
    local_user_id: user.local_user.id,
    email: email.to_string(),
    verification_token: uuid::Uuid::new_v4().to_string(),
  }
}

fn send_verification_link(
  local_site: &LocalSite,
  user: &LocalUserView,
  email: SensitiveString,
  form: &EmailVerificationForm,
  settings: &'static Settings,
) {
  let verify_link = format!(
    "{}/verify_email/{}",
    settings.get_protocol_and_hostname(),
    form.verification_token
  );
  let lang = user_language(&user.local_user);
  let subject = lang.verify_email_subject(&settings.hostname);

  // If an application is required, use a translation that includes that warning.
  let body = if local_site.registration_mode == RegistrationMode::RequireApplication {
    lang.verify_email_body_with_application(&settings.hostname, &user.person.name, verify_link)
  } else {
    lang.verify_email_body(&settings.hostname, &user.person.name, verify_link)
  };

  send_email(subject, email, user.person.name.clone(), body, settings);
}

/// Returns true if email was sent.
pub async fn send_verification_email_if_required(
  local_site: &LocalSite,
  user: &LocalUserView,
  pool: &mut DbPool<'_>,
  settings: &'static Settings,
) -> LemmyResult<bool> {
  if !user.local_user.admin
    && local_site.email_verification_required
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
  let lang = user_language(&user.local_user);
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
  let lang = user_language(&user.local_user);
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
  let lang = user_language(&user.local_user);
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
