#[macro_use]
extern crate diesel;
#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate lazy_static;

use chrono::NaiveDateTime;
use diesel::{result::Error, *};
use regex::Regex;
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

pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

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
  fn delete(_conn: &PgConnection, _id: i32) -> Result<usize, Error>
  where
    Self: Sized,
  {
    unimplemented!()
  }
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
  fn like(conn: &PgConnection, form: &T) -> Result<Self, Error>
  where
    Self: Sized;
  fn remove(conn: &PgConnection, user_id: i32, item_id: i32) -> Result<usize, Error>
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
  Active,
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
  Local,
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

pub fn is_email_regex(test: &str) -> bool {
  EMAIL_REGEX.is_match(test)
}

pub fn diesel_option_overwrite(opt: &Option<String>) -> Option<Option<String>> {
  match opt {
    // An empty string is an erase
    Some(unwrapped) => {
      if !unwrapped.eq("") {
        Some(Some(unwrapped.to_owned()))
      } else {
        Some(None)
      }
    }
    None => None,
  }
}

lazy_static! {
  static ref EMAIL_REGEX: Regex =
    Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$").unwrap();
}

#[cfg(test)]
mod tests {
  use super::fuzzy_search;
  use crate::{get_database_url_from_env, is_email_regex};
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

  #[test]
  fn test_email() {
    assert!(is_email_regex("gush@gmail.com"));
    assert!(!is_email_regex("nada_neutho"));
  }
}
