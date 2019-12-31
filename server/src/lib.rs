#![recursion_limit = "512"]
#[macro_use]
pub extern crate strum_macros;
#[macro_use]
pub extern crate lazy_static;
#[macro_use]
pub extern crate failure;
#[macro_use]
pub extern crate diesel;
pub extern crate actix;
pub extern crate actix_web;
pub extern crate bcrypt;
pub extern crate chrono;
pub extern crate dotenv;
pub extern crate jsonwebtoken;
pub extern crate lettre;
pub extern crate lettre_email;
pub extern crate rand;
pub extern crate regex;
pub extern crate serde;
pub extern crate serde_json;
pub extern crate sha2;
pub extern crate strum;

pub mod api;
pub mod apub;
pub mod db;
pub mod routes;
pub mod schema;
pub mod settings;
pub mod version;
pub mod websocket;

use crate::settings::Settings;
use chrono::{DateTime, NaiveDateTime, Utc};
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::extension::ClientId;
use lettre::smtp::ConnectionReuseParameters;
use lettre::{SmtpClient, Transport};
use lettre_email::Email;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use regex::{Regex, RegexBuilder};

pub fn to_datetime_utc(ndt: NaiveDateTime) -> DateTime<Utc> {
  DateTime::<Utc>::from_utc(ndt, Utc)
}

pub fn naive_now() -> NaiveDateTime {
  chrono::prelude::Utc::now().naive_utc()
}

pub fn naive_from_unix(time: i64) -> NaiveDateTime {
  NaiveDateTime::from_timestamp(time, 0)
}

pub fn is_email_regex(test: &str) -> bool {
  EMAIL_REGEX.is_match(test)
}

pub fn remove_slurs(test: &str) -> String {
  SLUR_REGEX.replace_all(test, "*removed*").to_string()
}

pub fn has_slurs(test: &str) -> bool {
  SLUR_REGEX.is_match(test)
}

pub fn extract_usernames(test: &str) -> Vec<&str> {
  let mut matches: Vec<&str> = USERNAME_MATCHES_REGEX
    .find_iter(test)
    .map(|mat| mat.as_str())
    .collect();

  // Unique
  matches.sort_unstable();
  matches.dedup();

  // Remove /u/
  matches.iter().map(|t| &t[3..]).collect()
}

pub fn generate_random_string() -> String {
  thread_rng().sample_iter(&Alphanumeric).take(30).collect()
}

pub fn send_email(
  subject: &str,
  to_email: &str,
  to_username: &str,
  html: &str,
) -> Result<(), String> {
  let email_config = Settings::get().email.as_ref().ok_or("no_email_setup")?;

  let email = Email::builder()
    .to((to_email, to_username))
    .from((
      email_config.smtp_login.to_owned(),
      email_config.smtp_from_address.to_owned(),
    ))
    .subject(subject)
    .html(html)
    .build()
    .unwrap();

  let mut mailer = SmtpClient::new_simple(&email_config.smtp_server)
    .unwrap()
    .hello_name(ClientId::Domain("localhost".to_string()))
    .credentials(Credentials::new(
      email_config.smtp_login.to_owned(),
      email_config.smtp_password.to_owned(),
    ))
    .smtp_utf8(true)
    .authentication_mechanism(Mechanism::Plain)
    .connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
    .transport();

  let result = mailer.send(email.into());

  match result {
    Ok(_) => Ok(()),
    Err(_) => Err("no_email_setup".to_string()),
  }
}

#[cfg(test)]
mod tests {
  use crate::{extract_usernames, has_slurs, is_email_regex, remove_slurs};

  #[test]
  fn test_email() {
    assert!(is_email_regex("gush@gmail.com"));
    assert!(!is_email_regex("nada_neutho"));
  }

  #[test]
  fn test_slur_filter() {
    let test =
      "coons test dindu ladyboy tranny retardeds. Capitalized Nigger. This is a bunch of other safe text.".to_string();
    let slur_free = "No slurs here";
    assert_eq!(
      remove_slurs(&test),
      "*removed* test *removed* *removed* *removed* *removed*. Capitalized *removed*. This is a bunch of other safe text."
        .to_string()
    );
    assert!(has_slurs(&test));
    assert!(!has_slurs(slur_free));
  }

  #[test]
  fn test_extract_usernames() {
    let usernames = extract_usernames("this is a user mention for [/u/testme](/u/testme) and thats all. Oh [/u/another](/u/another) user. And the first again [/u/testme](/u/testme) okay");
    let expected = vec!["another", "testme"];
    assert_eq!(usernames, expected);
  }

  // #[test]
  // fn test_send_email() {
  //  let result =  send_email("not a subject", "test_email@gmail.com", "ur user", "<h1>HI there</h1>");
  //   assert!(result.is_ok());
  // }
}

lazy_static! {
  static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$").unwrap();
  static ref SLUR_REGEX: Regex = RegexBuilder::new(r"(fag(g|got|tard)?|maricos?|cock\s?sucker(s|ing)?|nig(\b|g?(a|er)?s?)\b|dindu(s?)|mudslime?s?|kikes?|mongoloids?|towel\s*heads?|\bspi(c|k)s?\b|\bchinks?|niglets?|beaners?|\bnips?\b|\bcoons?\b|jungle\s*bunn(y|ies?)|jigg?aboo?s?|\bpakis?\b|rag\s*heads?|gooks?|cunts?|bitch(es|ing|y)?|puss(y|ies?)|twats?|feminazis?|whor(es?|ing)|\bslut(s|t?y)?|\btrann?(y|ies?)|ladyboy(s?)|\b(b|re|r)tard(ed)?s?)").case_insensitive(true).build().unwrap();
  static ref USERNAME_MATCHES_REGEX: Regex = Regex::new(r"/u/[a-zA-Z][0-9a-zA-Z_]*").unwrap();
}
