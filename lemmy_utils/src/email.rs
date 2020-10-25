use crate::settings::Settings;
use lettre::{
  message::{header, Mailbox, MultiPart, SinglePart},
  transport::smtp::{
    authentication::Credentials,
    client::{Tls, TlsParameters},
    extension::ClientId,
  },
  Address,
  Message,
  SmtpTransport,
  Transport,
};
use std::str::FromStr;

pub fn send_email(
  subject: &str,
  to_email: &str,
  to_username: &str,
  html: &str,
) -> Result<(), String> {
  let email_config = Settings::get().email.ok_or("no_email_setup")?;
  let domain = &Settings::get().hostname;

  let email = Message::builder()
    .from(
      email_config
        .smtp_from_address
        .parse()
        .expect("from address isn't valid"),
    )
    .to(Mailbox::new(
      Some(to_username.to_string()),
      Address::from_str(to_email).expect("to address isn't valid"),
    ))
    .subject(subject)
    .multipart(
      MultiPart::mixed().multipart(
        MultiPart::alternative()
          .singlepart(
            SinglePart::eight_bit()
              .header(header::ContentType(
                "text/plain; charset=utf8".parse().unwrap(),
              ))
              .body(html),
          )
          .multipart(
            MultiPart::related().singlepart(
              SinglePart::eight_bit()
                .header(header::ContentType(
                  "text/html; charset=utf8".parse().unwrap(),
                ))
                .body(html),
            ),
          ),
      ),
    )
    .expect("email built correctly");

  let mut builder = SmtpTransport::builder_dangerous(domain);

  // Set the TLS
  if email_config.use_tls {
    let tls_config = TlsParameters::new(domain.to_string()).expect("the TLS backend is happy");
    builder = builder.tls(Tls::Wrapper(tls_config));
  }

  // Set the creds if they exist
  if let (Some(username), Some(password)) = (email_config.smtp_login, email_config.smtp_password) {
    builder = builder.credentials(Credentials::new(username, password));
  }

  let mailer = builder
    .hello_name(ClientId::Domain(domain.to_string()))
    .build();

  let result = mailer.send(&email);

  match result {
    Ok(_) => Ok(()),
    Err(e) => Err(e.to_string()),
  }
}
