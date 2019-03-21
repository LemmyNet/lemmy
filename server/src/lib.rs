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


pub trait Crud<T> {
  fn create(conn: &PgConnection, form: T) -> Result<Self, Error> where Self: Sized;
  fn read(conn: &PgConnection, id: i32) -> Self;
  fn update(conn: &PgConnection, id: i32, form: T) -> Self;
  fn delete(conn: &PgConnection, id: i32) -> usize;
}

pub trait Followable<T> {
  fn follow(conn: &PgConnection, form: T) -> Result<Self, Error> where Self: Sized;
  fn ignore(conn: &PgConnection, form: T) -> usize;
}

pub trait Joinable<T> {
  fn join(conn: &PgConnection, form: T) -> Result<Self, Error> where Self: Sized;
  fn leave(conn: &PgConnection, form: T) -> usize;
}

pub trait Likeable<T> {
  fn like(conn: &PgConnection, form: T) -> Result<Self, Error> where Self: Sized;
  fn remove(conn: &PgConnection, form: T) -> usize;
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

#[cfg(test)]
mod tests {
  use Settings;
 #[test]
  fn test_api() {
    assert_eq!(Settings::get().api_endpoint(), "http://0.0.0.0/api/v1");
  }
} 
