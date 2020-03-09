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
use chttp::prelude::*;
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::extension::ClientId;
use lettre::smtp::ConnectionReuseParameters;
use lettre::{ClientSecurity, SmtpClient, Transport};
use lettre_email::Email;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use regex::{Regex, RegexBuilder};
use serde::Deserialize;

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

pub fn slur_check(test: &str) -> Result<(), Vec<&str>> {
  let mut matches: Vec<&str> = SLUR_REGEX.find_iter(test).map(|mat| mat.as_str()).collect();

  // Unique
  matches.sort_unstable();
  matches.dedup();

  if matches.is_empty() {
    Ok(())
  } else {
    Err(matches)
  }
}

pub fn slurs_vec_to_str(slurs: Vec<&str>) -> String {
  let start = "No slurs - ";
  let combined = &slurs.join(", ");
  [start, combined].concat()
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
    .from(email_config.smtp_from_address.to_owned())
    .subject(subject)
    .html(html)
    .build()
    .unwrap();

  let mailer = if email_config.use_tls {
    SmtpClient::new_simple(&email_config.smtp_server).unwrap()
  } else {
    SmtpClient::new(&email_config.smtp_server, ClientSecurity::None).unwrap()
  }
  .hello_name(ClientId::Domain(Settings::get().hostname.to_owned()))
  .smtp_utf8(true)
  .authentication_mechanism(Mechanism::Plain)
  .connection_reuse(ConnectionReuseParameters::ReuseUnlimited);
  let mailer = if let (Some(login), Some(password)) =
    (&email_config.smtp_login, &email_config.smtp_password)
  {
    mailer.credentials(Credentials::new(login.to_owned(), password.to_owned()))
  } else {
    mailer
  };

  let mut transport = mailer.transport();
  let result = transport.send(email.into());
  transport.close();

  match result {
    Ok(_) => Ok(()),
    Err(e) => Err(e.to_string()),
  }
}

#[derive(Deserialize, Debug)]
pub struct IframelyResponse {
  title: Option<String>,
  description: Option<String>,
  thumbnail_url: Option<String>,
  html: Option<String>,
}

pub fn fetch_iframely(url: &str) -> Result<IframelyResponse, failure::Error> {
  let fetch_url = format!("http://iframely/oembed?url={}", url);
  let text = chttp::get(&fetch_url)?.text()?;
  let res: IframelyResponse = serde_json::from_str(&text)?;
  Ok(res)
}

#[derive(Deserialize, Debug)]
pub struct PictshareResponse {
  status: String,
  url: String,
}

pub fn fetch_pictshare(image_url: &str) -> Result<PictshareResponse, failure::Error> {
  let fetch_url = format!(
    "http://pictshare/api/geturl.php?url={}",
    utf8_percent_encode(image_url, NON_ALPHANUMERIC)
  );
  let text = chttp::get(&fetch_url)?.text()?;
  let res: PictshareResponse = serde_json::from_str(&text)?;
  Ok(res)
}

fn fetch_iframely_and_pictshare_data(
  url: Option<String>,
) -> (
  Option<String>,
  Option<String>,
  Option<String>,
  Option<String>,
) {
  // Fetch iframely data
  let (iframely_title, iframely_description, iframely_thumbnail_url, iframely_html) = match url {
    Some(url) => match fetch_iframely(&url) {
      Ok(res) => (res.title, res.description, res.thumbnail_url, res.html),
      Err(e) => {
        eprintln!("iframely err: {}", e);
        (None, None, None, None)
      }
    },
    None => (None, None, None, None),
  };

  // Fetch pictshare thumbnail
  let pictshare_thumbnail = match iframely_thumbnail_url {
    Some(iframely_thumbnail_url) => match fetch_pictshare(&iframely_thumbnail_url) {
      Ok(res) => Some(res.url),
      Err(e) => {
        eprintln!("pictshare err: {}", e);
        None
      }
    },
    None => None,
  };

  (
    iframely_title,
    iframely_description,
    iframely_html,
    pictshare_thumbnail,
  )
}

#[cfg(test)]
mod tests {
  use crate::{extract_usernames, is_email_regex, remove_slurs, slur_check, slurs_vec_to_str};

  #[test]
  fn test_email() {
    assert!(is_email_regex("gush@gmail.com"));
    assert!(!is_email_regex("nada_neutho"));
  }

  #[test]
  fn test_slur_filter() {
    let test =
      "coons test dindu ladyboy tranny retardeds. Capitalized Niggerz. This is a bunch of other safe text.";
    let slur_free = "No slurs here";
    assert_eq!(
      remove_slurs(&test),
      "*removed* test *removed* *removed* *removed* *removed*. Capitalized *removed*. This is a bunch of other safe text."
        .to_string()
    );

    let has_slurs_vec = vec![
      "Niggerz",
      "coons",
      "dindu",
      "ladyboy",
      "retardeds",
      "tranny",
    ];
    let has_slurs_err_str = "No slurs - Niggerz, coons, dindu, ladyboy, retardeds, tranny";

    assert_eq!(slur_check(test), Err(has_slurs_vec));
    assert_eq!(slur_check(slur_free), Ok(()));
    if let Err(slur_vec) = slur_check(test) {
      assert_eq!(&slurs_vec_to_str(slur_vec), has_slurs_err_str);
    }
  }

  #[test]
  fn test_extract_usernames() {
    let usernames = extract_usernames("this is a user mention for [/u/testme](/u/testme) and thats all. Oh [/u/another](/u/another) user. And the first again [/u/testme](/u/testme) okay");
    let expected = vec!["another", "testme"];
    assert_eq!(usernames, expected);
  }

  // These helped with testing
  // #[test]
  // fn test_iframely() {
  //   let res = fetch_iframely("https://www.redspark.nu/?p=15341");
  //   assert!(res.is_ok());
  // }

  // #[test]
  // fn test_pictshare() {
  //   let res = fetch_pictshare("https://upload.wikimedia.org/wikipedia/en/2/27/The_Mandalorian_logo.jpg");
  //   assert!(res.is_ok());
  //   let res_other = fetch_pictshare("https://upload.wikimedia.org/wikipedia/en/2/27/The_Mandalorian_logo.jpgaoeu");
  //   assert!(res_other.is_err());
  // }

  // #[test]
  // fn test_send_email() {
  //  let result =  send_email("not a subject", "test_email@gmail.com", "ur user", "<h1>HI there</h1>");
  //   assert!(result.is_ok());
  // }
}

lazy_static! {
  static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$").unwrap();
  static ref SLUR_REGEX: Regex = RegexBuilder::new(r"(fag(g|got|tard)?|maricos?|cock\s?sucker(s|ing)?|nig(\b|g?(a|er)?(s|z)?)\b|dindu(s?)|mudslime?s?|kikes?|mongoloids?|towel\s*heads?|\bspi(c|k)s?\b|\bchinks?|niglets?|beaners?|\bnips?\b|\bcoons?\b|jungle\s*bunn(y|ies?)|jigg?aboo?s?|\bpakis?\b|rag\s*heads?|gooks?|cunts?|bitch(es|ing|y)?|puss(y|ies?)|twats?|feminazis?|whor(es?|ing)|\bslut(s|t?y)?|\btrann?(y|ies?)|ladyboy(s?)|\b(b|re|r)tard(ed)?s?)").case_insensitive(true).build().unwrap();
  static ref USERNAME_MATCHES_REGEX: Regex = Regex::new(r"/u/[a-zA-Z][0-9a-zA-Z_]*").unwrap();
}
