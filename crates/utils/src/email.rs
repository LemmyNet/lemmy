use crate::settings::structs::Settings;
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
  let email_config = Settings::get().email().ok_or("no_email_setup")?;
  let domain = Settings::get().hostname();

  let (smtp_server, smtp_port) = {
    let email_and_port = email_config.smtp_server.split(':').collect::<Vec<&str>>();
    (
      email_and_port[0],
      email_and_port[1]
        .parse::<u16>()
        .expect("email needs a port"),
    )
  };

  let email = Message::builder()
    .from(
      email_config
        .smtp_from_address
        .parse()
        .expect("email from address isn't valid"),
    )
    .to(Mailbox::new(
      Some(to_username.to_string()),
      Address::from_str(to_email).expect("email to address isn't valid"),
    ))
    .subject(subject)
    .multipart(
      MultiPart::mixed().multipart(
        MultiPart::alternative()
          .singlepart(
            SinglePart::builder()
              .header(header::ContentType(
                "text/plain; charset=utf8"
                  .parse()
                  .expect("email could not parse header"),
              ))
              .body(html.to_string()),
          )
          .multipart(
            MultiPart::related().singlepart(
              SinglePart::builder()
                .header(header::ContentType(
                  "text/html; charset=utf8"
                    .parse()
                    .expect("email could not parse header"),
                ))
                .body(html.to_string()),
            ),
          ),
      ),
    )
    .expect("email built incorrectly");

  // don't worry about 'dangeous'. it's just that leaving it at the default configuration
  // is bad.
  let mut builder = SmtpTransport::builder_dangerous(smtp_server).port(smtp_port);

  // Set the TLS
  if email_config.use_tls {
    let tls_config = TlsParameters::new(smtp_server.to_string()).expect("the TLS backend is happy");
    builder = builder.tls(Tls::Wrapper(tls_config));
  }

  // Set the creds if they exist
  if let (Some(username), Some(password)) = (email_config.smtp_login, email_config.smtp_password) {
    builder = builder.credentials(Credentials::new(username, password));
  }

  let mailer = builder.hello_name(ClientId::Domain(domain)).build();

  let result = mailer.send(&email);

  match result {
    Ok(_) => Ok(()),
    Err(e) => Err(e.to_string()),
  }
}
