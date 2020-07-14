#[macro_use]
pub extern crate diesel;
#[macro_use]
pub extern crate strum_macros;
pub extern crate bcrypt;
pub extern crate chrono;
pub extern crate log;
pub extern crate serde;
pub extern crate serde_json;
pub extern crate sha2;
pub extern crate strum;

use chrono::NaiveDateTime;
use diesel::{dsl::*, result::Error, *};
use serde::{Deserialize, Serialize};
use std::{env, env::VarError};

pub mod activity;
pub mod category;
pub mod comment;
pub mod comment_view;
pub mod community;
pub mod community_view;
pub mod moderator;
pub mod moderator_views;
pub mod password_reset_request;
pub mod post;
pub mod post_view;
pub mod private_message;
pub mod private_message_view;
pub mod schema;
pub mod site;
pub mod site_view;
pub mod user;
pub mod user_mention;
pub mod user_mention_view;
pub mod user_view;

pub trait Crud<T> {
  fn create(conn: &PgConnection, form: &T) -> Result<Self, Error>
  where
    Self: Sized;
  fn read(conn: &PgConnection, id: i32) -> Result<Self, Error>
  where
    Self: Sized;
  fn update(conn: &PgConnection, id: i32, form: &T) -> Result<Self, Error>
  where
    Self: Sized;
  fn delete(conn: &PgConnection, id: i32) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait Followable<T> {
  fn follow(conn: &PgConnection, form: &T) -> Result<Self, Error>
  where
    Self: Sized;
  fn unfollow(conn: &PgConnection, form: &T) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait Joinable<T> {
  fn join(conn: &PgConnection, form: &T) -> Result<Self, Error>
  where
    Self: Sized;
  fn leave(conn: &PgConnection, form: &T) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait Likeable<T> {
  fn read(conn: &PgConnection, id: i32) -> Result<Vec<Self>, Error>
  where
    Self: Sized;
  fn like(conn: &PgConnection, form: &T) -> Result<Self, Error>
  where
    Self: Sized;
  fn remove(conn: &PgConnection, form: &T) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait Bannable<T> {
  fn ban(conn: &PgConnection, form: &T) -> Result<Self, Error>
  where
    Self: Sized;
  fn unban(conn: &PgConnection, form: &T) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait Saveable<T> {
  fn save(conn: &PgConnection, form: &T) -> Result<Self, Error>
  where
    Self: Sized;
  fn unsave(conn: &PgConnection, form: &T) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait Readable<T> {
  fn mark_as_read(conn: &PgConnection, form: &T) -> Result<Self, Error>
  where
    Self: Sized;
  fn mark_as_unread(conn: &PgConnection, form: &T) -> Result<usize, Error>
  where
    Self: Sized;
}

pub trait MaybeOptional<T> {
  fn get_optional(self) -> Option<T>;
}

impl<T> MaybeOptional<T> for T {
  fn get_optional(self) -> Option<T> {
    Some(self)
  }
}

impl<T> MaybeOptional<T> for Option<T> {
  fn get_optional(self) -> Option<T> {
    self
  }
}

pub fn get_database_url_from_env() -> Result<String, VarError> {
  env::var("LEMMY_DATABASE_URL")
}

#[derive(EnumString, ToString, Debug, Serialize, Deserialize)]
pub enum SortType {
  Hot,
  New,
  TopDay,
  TopWeek,
  TopMonth,
  TopYear,
  TopAll,
}

#[derive(EnumString, ToString, Debug, Serialize, Deserialize)]
pub enum ListingType {
  All,
  Subscribed,
  Community,
}

#[derive(EnumString, ToString, Debug, Serialize, Deserialize)]
pub enum SearchType {
  All,
  Comments,
  Posts,
  Communities,
  Users,
  Url,
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

pub fn naive_now() -> NaiveDateTime {
  chrono::prelude::Utc::now().naive_utc()
}

#[cfg(test)]
mod tests {
  use super::fuzzy_search;
  use crate::get_database_url_from_env;
  use diesel::{Connection, PgConnection};

  pub fn establish_unpooled_connection() -> PgConnection {
    let db_url = match get_database_url_from_env() {
      Ok(url) => url,
      Err(e) => panic!(
        "Failed to read database URL from env var LEMMY_DATABASE_URL: {}",
        e
      ),
    };
    PgConnection::establish(&db_url).unwrap_or_else(|_| panic!("Error connecting to {}", db_url))
  }

  #[test]
  fn test_fuzzy_search() {
    let test = "This is a fuzzy search";
    assert_eq!(fuzzy_search(test), "%This%is%a%fuzzy%search%".to_string());
  }
}
