// Avoid warnings for unused 0.19 translations
#![allow(dead_code)]

use lemmy_db_schema::sensitive::SensitiveString;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
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
use translations::Lang;
use uuid::Uuid;

pub mod account;
pub mod admin;
pub mod notifications;
mod translations {
  rosetta_i18n::include_translations!();
}

type AsyncSmtpTransport = lettre::AsyncSmtpTransport<lettre::Tokio1Executor>;

fn inbox_link(settings: &Settings) -> String {
  format!("{}/inbox", settings.get_protocol_and_hostname())
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
fn user_language(local_user_view: &LocalUserView) -> Lang {
  let lang_id = LanguageId::new(&local_user_view.local_user.interface_language);
  Lang::from_language_id(&lang_id).unwrap_or_else(|| {
    let en = LanguageId::new("en");
    Lang::from_language_id(&en).expect("default language")
  })
}

fn user_email(local_user_view: &LocalUserView) -> LemmyResult<SensitiveString> {
  local_user_view
    .local_user
    .email
    .clone()
    .ok_or(LemmyErrorType::EmailRequired.into())
}
