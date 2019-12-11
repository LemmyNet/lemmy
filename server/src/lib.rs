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
pub extern crate crypto;
pub extern crate dotenv;
pub extern crate jsonwebtoken;
pub extern crate lettre;
pub extern crate lettre_email;
pub extern crate rand;
pub extern crate regex;
pub extern crate serde;
pub extern crate serde_json;
pub extern crate strum;

pub mod api;
pub mod apub;
pub mod db;
pub mod feeds;
pub mod nodeinfo;
pub mod schema;
pub mod version;
pub mod websocket;

use chrono::{DateTime, NaiveDateTime, Utc};
use dotenv::dotenv;
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::extension::ClientId;
use lettre::smtp::ConnectionReuseParameters;
use lettre::{SmtpClient, Transport};
use lettre_email::Email;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use regex::Regex;
use std::env;
use std::net::IpAddr;

pub struct Settings {
  pub db_url: String,
  pub hostname: String,
  pub bind: IpAddr,
  pub port: u16,
  pub jwt_secret: String,
  pub rate_limit_message: i32,
  pub rate_limit_message_per_second: i32,
  pub rate_limit_post: i32,
  pub rate_limit_post_per_second: i32,
  pub rate_limit_register: i32,
  pub rate_limit_register_per_second: i32,
  pub email_config: Option<EmailConfig>,
}

pub struct EmailConfig {
  smtp_server: String,
  smtp_login: String,
  smtp_password: String,
  smtp_from_address: String,
}

impl Settings {
  pub fn get() -> Self {
    dotenv().ok();

    let email_config =
      if env::var("SMTP_SERVER").is_ok() && !env::var("SMTP_SERVER").unwrap().eq("") {
        Some(EmailConfig {
          smtp_server: env::var("SMTP_SERVER").expect("SMTP_SERVER must be set"),
          smtp_login: env::var("SMTP_LOGIN").expect("SMTP_LOGIN must be set"),
          smtp_password: env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set"),
          smtp_from_address: env::var("SMTP_FROM_ADDRESS").expect("SMTP_FROM_ADDRESS must be set"),
        })
      } else {
        None
      };

    Settings {
      db_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
      hostname: env::var("HOSTNAME").unwrap_or("rrr".to_string()),
      bind: env::var("BIND")
        .unwrap_or("0.0.0.0".to_string())
        .parse()
        .unwrap(),
      port: env::var("PORT")
        .unwrap_or("8536".to_string())
        .parse()
        .unwrap(),
      jwt_secret: env::var("JWT_SECRET").unwrap_or("changeme".to_string()),
      rate_limit_message: env::var("RATE_LIMIT_MESSAGE")
        .unwrap_or("30".to_string())
        .parse()
        .unwrap(),
      rate_limit_message_per_second: env::var("RATE_LIMIT_MESSAGE_PER_SECOND")
        .unwrap_or("60".to_string())
        .parse()
        .unwrap(),
      rate_limit_post: env::var("RATE_LIMIT_POST")
        .unwrap_or("3".to_string())
        .parse()
        .unwrap(),
      rate_limit_post_per_second: env::var("RATE_LIMIT_POST_PER_SECOND")
        .unwrap_or("600".to_string())
        .parse()
        .unwrap(),
      rate_limit_register: env::var("RATE_LIMIT_REGISTER")
        .unwrap_or("1".to_string())
        .parse()
        .unwrap(),
      rate_limit_register_per_second: env::var("RATE_LIMIT_REGISTER_PER_SECOND")
        .unwrap_or("3600".to_string())
        .parse()
        .unwrap(),
      email_config,
    }
  }
  fn api_endpoint(&self) -> String {
    format!("{}/api/v1", self.hostname)
  }
}

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
  let email_config = Settings::get().email_config.ok_or("no_email_setup")?;

  let email = Email::builder()
    .to((to_email, to_username))
    .from((
      email_config.smtp_login.to_owned(),
      email_config.smtp_from_address,
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
  use crate::{extract_usernames, has_slurs, is_email_regex, remove_slurs, Settings};
  #[test]
  fn test_api() {
    assert_eq!(Settings::get().api_endpoint(), "rrr/api/v1");
  }

  #[test]
  fn test_email() {
    assert!(is_email_regex("gush@gmail.com"));
    assert!(!is_email_regex("nada_neutho"));
  }

  #[test]
  fn test_slur_filter() {
    let test =
      "coons test dindu ladyboy tranny retardeds. This is a bunch of other safe text.".to_string();
    let slur_free = "No slurs here";
    assert_eq!(
      remove_slurs(&test),
      "*removed* test *removed* *removed* *removed* *removed*. This is a bunch of other safe text."
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
  static ref SLUR_REGEX: Regex = Regex::new(r"(fag(g|got|tard)?|maricos?|cock\s?sucker(s|ing)?|nig(\b|g?(a|er)?s?)\b|dindu(s?)|mudslime?s?|kikes?|mongoloids?|towel\s*heads?|\bspi(c|k)s?\b|\bchinks?|niglets?|beaners?|\bnips?\b|\bcoons?\b|jungle\s*bunn(y|ies?)|jigg?aboo?s?|\bpakis?\b|rag\s*heads?|gooks?|cunts?|bitch(es|ing|y)?|puss(y|ies?)|twats?|feminazis?|whor(es?|ing)|\bslut(s|t?y)?|\btrann?(y|ies?)|ladyboy(s?)|\b(b|re|r)tard(ed)?s?)").unwrap();
  static ref USERNAME_MATCHES_REGEX: Regex = Regex::new(r"/u/[a-zA-Z][0-9a-zA-Z_]*").unwrap();
}
