#[macro_use]
pub extern crate diesel;
pub extern crate dotenv;
pub extern crate chrono;
pub extern crate serde;
pub extern crate serde_json;
pub extern crate actix;
pub extern crate actix_web;
pub extern crate rand;
pub extern crate strum;
pub extern crate jsonwebtoken;
pub extern crate bcrypt;
pub extern crate regex;
#[macro_use] pub extern crate strum_macros;
#[macro_use] pub extern crate lazy_static;
#[macro_use] extern crate failure;

pub mod schema;
pub mod apub;
pub mod actions;
pub mod websocket_server;

use diesel::*;
use diesel::pg::PgConnection;
use diesel::result::Error;
use dotenv::dotenv;
use std::env;
use regex::Regex;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDateTime, Utc};

pub trait Crud<T> {
  fn create(conn: &PgConnection, form: &T) -> Result<Self, Error> where Self: Sized;
  fn read(conn: &PgConnection, id: i32) -> Result<Self, Error> where Self: Sized;  
  fn update(conn: &PgConnection, id: i32, form: &T) -> Result<Self, Error> where Self: Sized;  
  fn delete(conn: &PgConnection, id: i32) -> Result<usize, Error> where Self: Sized;
}

pub trait Followable<T> {
  fn follow(conn: &PgConnection, form: &T) -> Result<Self, Error> where Self: Sized;
  fn ignore(conn: &PgConnection, form: &T) -> Result<usize, Error> where Self: Sized;
}

pub trait Joinable<T> {
  fn join(conn: &PgConnection, form: &T) -> Result<Self, Error> where Self: Sized;
  fn leave(conn: &PgConnection, form: &T) -> Result<usize, Error> where Self: Sized;
}

pub trait Likeable<T> {
  fn read(conn: &PgConnection, id: i32) -> Result<Vec<Self>, Error> where Self: Sized;
  fn like(conn: &PgConnection, form: &T) -> Result<Self, Error> where Self: Sized;
  fn remove(conn: &PgConnection, form: &T) -> Result<usize, Error> where Self: Sized;
}

pub trait Bannable<T> {
  fn ban(conn: &PgConnection, form: &T) -> Result<Self, Error> where Self: Sized;
  fn unban(conn: &PgConnection, form: &T) -> Result<usize, Error> where Self: Sized;
}

pub trait Saveable<T> {
  fn save(conn: &PgConnection, form: &T) -> Result<Self, Error> where Self: Sized;
  fn unsave(conn: &PgConnection, form: &T) -> Result<usize, Error> where Self: Sized;
}

pub trait Readable<T> {
  fn mark_as_read(conn: &PgConnection, form: &T) -> Result<Self, Error> where Self: Sized;
  fn mark_as_unread(conn: &PgConnection, form: &T) -> Result<usize, Error> where Self: Sized;
}

pub fn establish_connection() -> PgConnection {
  let db_url = Settings::get().db_url;
  PgConnection::establish(&db_url)
    .expect(&format!("Error connecting to {}", db_url))
}

pub struct Settings {
  db_url: String,
  hostname: String,
  jwt_secret: String,
}

impl Settings {
  fn get() -> Self {
    dotenv().ok();
    Settings {
      db_url: env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set"),
        hostname: env::var("HOSTNAME").unwrap_or("rrr".to_string()),
        jwt_secret: env::var("JWT_SECRET").unwrap_or("changeme".to_string()),
    }
  }
  fn api_endpoint(&self) -> String {
    format!("{}/api/v1", self.hostname)
  }
}

#[derive(EnumString,ToString,Debug, Serialize, Deserialize)]
pub enum SortType {
  Hot, New, TopDay, TopWeek, TopMonth, TopYear, TopAll
}

#[derive(EnumString,ToString,Debug, Serialize, Deserialize)]
pub enum SearchType {
  Both, Comments, Posts
}

pub fn to_datetime_utc(ndt: NaiveDateTime) -> DateTime<Utc> {
  DateTime::<Utc>::from_utc(ndt, Utc)
}

pub fn naive_now() -> NaiveDateTime {
  chrono::prelude::Utc::now().naive_utc()
}

pub fn naive_from_unix(time: i64)  ->  NaiveDateTime {
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

pub fn fuzzy_search(q: &str) -> String {
  let replaced = q.replace(" ", "%");
  format!("%{}%", replaced)
}

pub fn limit_and_offset(page: Option<i64>, limit: Option<i64>) -> (i64, i64) {
    let page = page.unwrap_or(1);
    let limit = limit.unwrap_or(10);
    let offset = limit * (page - 1);
    (limit, offset)
}

#[cfg(test)]
mod tests {
  use {Settings, is_email_regex, remove_slurs, has_slurs, fuzzy_search};
  #[test]
  fn test_api() {
    assert_eq!(Settings::get().api_endpoint(), "rrr/api/v1");
  }

  #[test] fn test_email() {
    assert!(is_email_regex("gush@gmail.com"));
    assert!(!is_email_regex("nada_neutho"));
  } 

  #[test] fn test_slur_filter() {
    let test = "coons test dindu ladyboy tranny. This is a bunch of other safe text.".to_string();
    let slur_free = "No slurs here";
    assert_eq!(remove_slurs(&test), "*removed* test *removed* *removed* *removed*. This is a bunch of other safe text.".to_string());
    assert!(has_slurs(&test));
    assert!(!has_slurs(slur_free));
  } 

  #[test] fn test_fuzzy_search() {
    let test = "This is a fuzzy search";
    assert_eq!(fuzzy_search(test), "%This%is%a%fuzzy%search%".to_string());
  }
}



lazy_static! {
  static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$").unwrap();
  static ref SLUR_REGEX: Regex = Regex::new(r"(fag(g|got|tard)?|maricos?|cock\s?sucker(s|ing)?|\bnig(\b|g?(a|er)?s?)\b|dindu(s?)|mudslime?s?|kikes?|mongoloids?|towel\s*heads?|\bspi(c|k)s?\b|\bchinks?|niglets?|beaners?|\bnips?\b|\bcoons?\b|jungle\s*bunn(y|ies?)|jigg?aboo?s?|\bpakis?\b|rag\s*heads?|gooks?|cunts?|bitch(es|ing|y)?|puss(y|ies?)|twats?|feminazis?|whor(es?|ing)|\bslut(s|t?y)?|\btrann?(y|ies?)|ladyboy(s?))").unwrap();
}

