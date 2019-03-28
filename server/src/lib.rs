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

pub fn establish_connection() -> PgConnection {
  let db_url = Settings::get().db_url;
  PgConnection::establish(&db_url)
    .expect(&format!("Error connecting to {}", db_url))
}

pub struct Settings {
  db_url: String,
  hostname: String
}

impl Settings {
  fn get() -> Self {
    dotenv().ok();
    Settings {
      db_url: env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set"),
        hostname: env::var("HOSTNAME").unwrap_or("http://0.0.0.0".to_string())
    }
  }
  fn api_endpoint(&self) -> String {
    format!("{}/api/v1", self.hostname)
  }
}

use chrono::{DateTime, NaiveDateTime, Utc};
pub fn to_datetime_utc(ndt: NaiveDateTime) -> DateTime<Utc> {
  DateTime::<Utc>::from_utc(ndt, Utc)
}

pub fn naive_now() -> NaiveDateTime {
  chrono::prelude::Utc::now().naive_utc()
}

pub fn is_email_regex(test: &str) -> bool {
  let re = Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$").unwrap();
  re.is_match(test)
}

#[cfg(test)]
mod tests {
  use {Settings, is_email_regex};
  #[test]
  fn test_api() {
    assert_eq!(Settings::get().api_endpoint(), "http://0.0.0.0/api/v1");
  }

  #[test]
  fn test_email() {
    assert!(is_email_regex("gush@gmail.com"));
    assert!(!is_email_regex("nada_neutho"));
  } 
}
