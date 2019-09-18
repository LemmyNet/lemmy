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
pub extern crate rand;
pub extern crate regex;
pub extern crate serde;
pub extern crate serde_json;
pub extern crate strum;

pub mod api;
pub mod apub;
pub mod db;
pub mod schema;
pub mod websocket;

use chrono::{DateTime, NaiveDateTime, Utc};
use dotenv::dotenv;
use regex::Regex;
use std::env;

pub struct Settings {
  db_url: String,
  hostname: String,
  jwt_secret: String,
}

impl Settings {
  fn get() -> Self {
    dotenv().ok();
    Settings {
      db_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
      hostname: env::var("HOSTNAME").unwrap_or("rrr".to_string()),
      jwt_secret: env::var("JWT_SECRET").unwrap_or("changeme".to_string()),
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

#[cfg(test)]
mod tests {
  use crate::{has_slurs, is_email_regex, remove_slurs, Settings};
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
    let test = "coons test dindu ladyboy tranny retardeds. This is a bunch of other safe text.".to_string();
    let slur_free = "No slurs here";
    assert_eq!(
      remove_slurs(&test),
      "*removed* test *removed* *removed* *removed* *removed*. This is a bunch of other safe text."
        .to_string()
    );
    assert!(has_slurs(&test));
    assert!(!has_slurs(slur_free));
  }

}

lazy_static! {
  static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$").unwrap();
  static ref SLUR_REGEX: Regex = Regex::new(r"(fag(g|got|tard)?|maricos?|cock\s?sucker(s|ing)?|\bnig(\b|g?(a|er)?s?)\b|dindu(s?)|mudslime?s?|kikes?|mongoloids?|towel\s*heads?|\bspi(c|k)s?\b|\bchinks?|niglets?|beaners?|\bnips?\b|\bcoons?\b|jungle\s*bunn(y|ies?)|jigg?aboo?s?|\bpakis?\b|rag\s*heads?|gooks?|cunts?|bitch(es|ing|y)?|puss(y|ies?)|twats?|feminazis?|whor(es?|ing)|\bslut(s|t?y)?|\btrann?(y|ies?)|ladyboy(s?)|\b(b|re|r)tard(ed)?s?)").unwrap();
}
