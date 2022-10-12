use crate::error::LemmyError;
use html2text;
use lettre::{
  message::{Mailbox, MultiPart},
  transport::smtp::{authentication::Credentials, extension::ClientId},
  Address,
  Message,
  SmtpTransport,
  Transport,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

pub mod translations {
  rosetta_i18n::include_translations!();
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmailConfig {
  /// Hostname and port of the smtp server
  /// example = "localhost:25"
  pub smtp_server: String,
  /// Login name for smtp server
  pub smtp_login: Option<String>,
  /// Password to login to the smtp server
  pub smtp_password: Option<String>,
  /// example = "noreply@example.com"
  /// Address to send emails from, eg "noreply@your-instance.com"
  pub smtp_from_address: Option<String>,
  /// Whether or not smtp connections should use tls. Can be none, tls, or starttls
  /// example = "none"
  pub tls_type: String,
}

pub fn send_email(
  subject: &str,
  to_email: &str,
  to_username: &str,
  html: &str,
  hostname: &str,
  config: &EmailConfig,
) -> Result<(), LemmyError> {
  let (smtp_server, smtp_port) = {
    let email_and_port = config.smtp_server.split(':').collect::<Vec<&str>>();
    if email_and_port.len() == 1 {
      return Err(LemmyError::from_message(
        "email.smtp_server needs a port, IE smtp.xxx.com:465",
      ));
    }

    (
      email_and_port[0],
      email_and_port[1]
        .parse::<u16>()
        .expect("email needs a port"),
    )
  };

  // the message length before wrap, 78, is somewhat arbritary but looks good to me
  let plain_text = html2text::from_read(html.as_bytes(), 78);

  let email = Message::builder()
    .from(
      config
        .smtp_from_address
        .as_ref()
        .expect("email from address isn't valid")
        .parse()
        .expect("email from address isn't valid"),
    )
    .to(Mailbox::new(
      Some(to_username.to_string()),
      Address::from_str(to_email).expect("email to address isn't valid"),
    ))
    .message_id(Some(format!("{}@{}", Uuid::new_v4(), hostname)))
    .subject(subject)
    .multipart(MultiPart::alternative_plain_html(
      plain_text,
      html.to_string(),
    ))
    .expect("email built incorrectly");

  // don't worry about 'dangeous'. it's just that leaving it at the default configuration
  // is bad.

  // Set the TLS
  let builder_dangerous = SmtpTransport::builder_dangerous(smtp_server).port(smtp_port);

  let mut builder = match config.tls_type.as_str() {
    "starttls" => SmtpTransport::starttls_relay(smtp_server)?,
    "tls" => SmtpTransport::relay(smtp_server)?,
    _ => builder_dangerous,
  };

  // Set the creds if they exist
  if let (Some(username), Some(password)) = (
    config.smtp_login.to_owned(),
    config.smtp_password.to_owned(),
  ) {
    builder = builder.credentials(Credentials::new(username, password));
  }

  let mailer = builder
    .hello_name(ClientId::Domain(hostname.to_owned()))
    .build();

  let result = mailer.send(&email);

  match result {
    Ok(_) => Ok(()),
    Err(e) => Err(LemmyError::from_error_message(e, "email_send_failed")),
  }
}
