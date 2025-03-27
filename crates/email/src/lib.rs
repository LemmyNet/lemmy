use lemmy_db_schema::{
  newtypes::DbUrl,
  source::{
    comment::Comment,
    email_verification::{EmailVerification, EmailVerificationForm},
    local_site::LocalSite,
    local_user::LocalUser,
    password_reset_request::PasswordResetRequest,
    person::Person,
    post::Post,
  },
  utils::DbPool,
  RegistrationMode,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
  utils::markdown::markdown_to_html,
};
use lettre::{
  message::{Mailbox, MultiPart},
  transport::smtp::extension::ClientId,
  Address,
  AsyncTransport,
  Message,
};
use rosetta_i18n::{Language, LanguageId};
use std::{str::FromStr, sync::OnceLock};
use tracing::warn;
use translations::Lang;
use uuid::Uuid;

pub mod translations {
  rosetta_i18n::include_translations!();
}

type AsyncSmtpTransport = lettre::AsyncSmtpTransport<lettre::Tokio1Executor>;

fn inbox_link(settings: &Settings) -> String {
  format!("{}/inbox", settings.get_protocol_and_hostname())
}

pub async fn send_mention_email(
  mention_user_view: &LocalUserView,
  content: &str,
  person: &Person,
  link: DbUrl,
  settings: &Settings,
) {
  let inbox_link = inbox_link(settings);
  let lang = user_language(&mention_user_view.local_user);
  let content = markdown_to_html(content);
  send_email_to_user(
    mention_user_view,
    &lang.notification_mentioned_by_subject(&person.name),
    &lang.notification_mentioned_by_body(&link, &content, &inbox_link, &person.name),
    settings,
  )
  .await
}

pub async fn send_comment_reply_email(
  parent_user_view: &LocalUserView,
  comment: &Comment,
  person: &Person,
  parent_comment: &Comment,
  post: &Post,
  settings: &Settings,
) -> LemmyResult<()> {
  let inbox_link = inbox_link(settings);
  let lang = user_language(&parent_user_view.local_user);
  let content = markdown_to_html(&comment.content);
  send_email_to_user(
    parent_user_view,
    &lang.notification_comment_reply_subject(&person.name),
    &lang.notification_comment_reply_body(
      comment.local_url(settings)?,
      &content,
      &inbox_link,
      &parent_comment.content,
      &post.name,
      &person.name,
    ),
    settings,
  )
  .await;
  Ok(())
}

pub async fn send_post_reply_email(
  parent_user_view: &LocalUserView,
  comment: &Comment,
  person: &Person,
  post: &Post,
  settings: &Settings,
) -> LemmyResult<()> {
  let inbox_link = inbox_link(settings);
  let lang = user_language(&parent_user_view.local_user);
  let content = markdown_to_html(&comment.content);
  send_email_to_user(
    parent_user_view,
    &lang.notification_post_reply_subject(&person.name),
    &lang.notification_post_reply_body(
      comment.local_url(settings)?,
      &content,
      &inbox_link,
      &post.name,
      &person.name,
    ),
    settings,
  )
  .await;
  Ok(())
}

async fn send_email_to_user(
  local_user_view: &LocalUserView,
  subject: &str,
  body: &str,
  settings: &Settings,
) {
  if local_user_view.person.banned || !local_user_view.local_user.send_notifications_to_email {
    return;
  }

  if let Some(user_email) = &local_user_view.local_user.email {
    match send_email(
      subject,
      user_email,
      &local_user_view.person.name,
      body,
      settings,
    )
    .await
    {
      Ok(_o) => _o,
      Err(e) => warn!("{}", e),
    };
  }
}

pub async fn send_password_reset_email(
  user: &LocalUserView,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> LemmyResult<()> {
  // Generate a random token
  let token = uuid::Uuid::new_v4().to_string();

  let email = &user
    .local_user
    .email
    .clone()
    .ok_or(LemmyErrorType::EmailRequired)?;
  let lang = user_language(&user.local_user);
  let subject = &lang.password_reset_subject(&user.person.name);
  let protocol_and_hostname = settings.get_protocol_and_hostname();
  let reset_link = format!("{}/password_change/{}", protocol_and_hostname, &token);
  let body = &lang.password_reset_body(reset_link, &user.person.name);
  send_email(subject, email, &user.person.name, body, settings).await?;

  // Insert the row after successful send, to avoid using daily reset limit while
  // email sending is broken.
  let local_user_id = user.local_user.id;
  PasswordResetRequest::create(pool, local_user_id, token.clone()).await?;
  Ok(())
}

pub async fn send_private_message_email(
  sender: &LocalUserView,
  local_recipient: &LocalUserView,
  content: &str,
  settings: &Settings,
) {
  let inbox_link = inbox_link(settings);
  let lang = user_language(&local_recipient.local_user);
  let sender_name = &sender.person.name;
  let content = markdown_to_html(content);
  send_email_to_user(
    local_recipient,
    &lang.notification_private_message_subject(sender_name),
    &lang.notification_private_message_body(inbox_link, &content, sender_name),
    settings,
  )
  .await;
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

/// Send a new applicant email notification to all admins
pub async fn send_new_applicant_email_to_admins(
  applicant_username: &str,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> LemmyResult<()> {
  // Collect the admins with emails
  let admins = LocalUserView::list_admins_with_emails(pool).await?;

  let applications_link = &format!(
    "{}/registration_applications",
    settings.get_protocol_and_hostname(),
  );

  for admin in &admins {
    let email = &admin
      .local_user
      .email
      .clone()
      .ok_or(LemmyErrorType::EmailRequired)?;
    let lang = user_language(&admin.local_user);
    let subject = lang.new_application_subject(&settings.hostname, applicant_username);
    let body = lang.new_application_body(applications_link);
    send_email(&subject, email, &admin.person.name, &body, settings).await?;
  }
  Ok(())
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

/// Send a report to all admins
pub async fn send_new_report_email_to_admins(
  reporter_username: &str,
  reported_username: &str,
  pool: &mut DbPool<'_>,
  settings: &Settings,
) -> LemmyResult<()> {
  // Collect the admins with emails
  let admins = LocalUserView::list_admins_with_emails(pool).await?;

  let reports_link = &format!("{}/reports", settings.get_protocol_and_hostname(),);

  for admin in &admins {
    if let Some(email) = &admin.local_user.email {
      let lang = user_language(&admin.local_user);
      let subject =
        lang.new_report_subject(&settings.hostname, reported_username, reporter_username);
      let body = lang.new_report_body(reports_link);
      send_email(&subject, email, &admin.person.name, &body, settings).await?;
    }
  }
  Ok(())
}

async fn send_email(
  subject: &str,
  to_email: &str,
  to_username: &str,
  html: &str,
  settings: &Settings,
) -> LemmyResult<()> {
  static MAILER: OnceLock<AsyncSmtpTransport> = OnceLock::new();
  let email_config = settings.email.clone().ok_or(LemmyErrorType::NoEmailSetup)?;

  #[expect(clippy::expect_used)]
  let mailer = MAILER.get_or_init(|| {
    AsyncSmtpTransport::from_url(&email_config.connection)
      .expect("init email transport")
      .hello_name(ClientId::Domain(settings.hostname.clone()))
      .build()
  });

  // use usize::MAX as the line wrap length, since lettre handles the wrapping for us
  let plain_text = html2text::from_read(html.as_bytes(), usize::MAX)?;

  let smtp_from_address = &email_config.smtp_from_address;

  let email = Message::builder()
    .from(
      smtp_from_address
        .parse()
        .with_lemmy_type(LemmyErrorType::InvalidEmailAddress(
          smtp_from_address.into(),
        ))?,
    )
    .to(Mailbox::new(
      Some(to_username.to_string()),
      Address::from_str(to_email)
        .with_lemmy_type(LemmyErrorType::InvalidEmailAddress(to_email.into()))?,
    ))
    .message_id(Some(format!("<{}@{}>", Uuid::new_v4(), settings.hostname)))
    .subject(subject)
    .multipart(MultiPart::alternative_plain_html(
      plain_text,
      html.to_string(),
    ))
    .with_lemmy_type(LemmyErrorType::EmailSendFailed)?;

  mailer
    .send(email)
    .await
    .with_lemmy_type(LemmyErrorType::EmailSendFailed)?;

  Ok(())
}

#[allow(clippy::expect_used)]
fn user_language(local_user: &LocalUser) -> Lang {
  let lang_id = LanguageId::new(&local_user.interface_language);
  Lang::from_language_id(&lang_id).unwrap_or_else(|| {
    let en = LanguageId::new("en");
    Lang::from_language_id(&en).expect("default language")
  })
}
