#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_newtype;
#[macro_use]
extern crate lazy_static;
// this is used in tests
#[allow(unused_imports)]
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate strum_macros;

pub mod aggregates;
pub mod impls;
pub mod newtypes;
pub mod schema;
pub mod source;
pub mod traits;

use crate::newtypes::DbUrl;
use chrono::NaiveDateTime;
use diesel::{Connection, PgConnection};
use lemmy_utils::ApiError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{env, env::VarError};
use url::Url;

pub type DbPool = deadpool_diesel::postgres::Pool;

pub fn get_database_url_from_env() -> Result<String, VarError> {
  env::var("LEMMY_DATABASE_URL")
}

#[derive(EnumString, ToString, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SortType {
  Active,
  Hot,
  New,
  TopDay,
  TopWeek,
  TopMonth,
  TopYear,
  TopAll,
  MostComments,
  NewComments,
}

#[derive(EnumString, ToString, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ListingType {
  All,
  Local,
  Subscribed,
  Community,
}

#[derive(EnumString, ToString, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SearchType {
  All,
  Comments,
  Posts,
  Communities,
  Users,
  Url,
}

pub fn from_opt_str_to_opt_enum<T: std::str::FromStr>(opt: &Option<String>) -> Option<T> {
  opt.as_ref().map(|t| T::from_str(t).ok()).flatten()
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

pub fn diesel_option_overwrite_to_url(
  opt: &Option<String>,
) -> Result<Option<Option<DbUrl>>, ApiError> {
  match opt.as_ref().map(|s| s.as_str()) {
    // An empty string is an erase
    Some("") => Ok(Some(None)),
    Some(str_url) => match Url::parse(str_url) {
      Ok(url) => Ok(Some(Some(url.into()))),
      Err(e) => Err(ApiError::err("invalid_url", e)),
    },
    None => Ok(None),
  }
}

embed_migrations!();

pub fn establish_unpooled_connection() -> PgConnection {
  let db_url = match get_database_url_from_env() {
    Ok(url) => url,
    Err(e) => panic!(
      "Failed to read database URL from env var LEMMY_DATABASE_URL: {}",
      e
    ),
  };
  let conn =
    PgConnection::establish(&db_url).unwrap_or_else(|_| panic!("Error connecting to {}", db_url));
  embedded_migrations::run(&conn).expect("load migrations");
  conn
}

pub fn establish_unpooled_connection_with_db_url(db_url: &str) -> PgConnection {
  let conn =
    PgConnection::establish(db_url).unwrap_or_else(|_| panic!("Error connecting to {}", db_url));
  embedded_migrations::run(&conn).expect("load migrations");
  conn
}

pub fn naive_now() -> NaiveDateTime {
  chrono::prelude::Utc::now().naive_utc()
}

lazy_static! {
  static ref EMAIL_REGEX: Regex =
    Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$")
      .expect("compile email regex");
}

pub mod functions {
  use diesel::sql_types::*;

  sql_function! {
    fn hot_rank(score: BigInt, time: Timestamp) -> Integer;
  }
}

#[cfg(test)]
mod tests {
  use super::{fuzzy_search, *};
  use crate::is_email_regex;

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

  #[test]
  fn test_diesel_option_overwrite() {
    assert_eq!(diesel_option_overwrite(&None), None);
    assert_eq!(diesel_option_overwrite(&Some("".to_string())), Some(None));
    assert_eq!(
      diesel_option_overwrite(&Some("test".to_string())),
      Some(Some("test".to_string()))
    );
  }

  #[test]
  fn test_diesel_option_overwrite_to_url() {
    assert!(matches!(diesel_option_overwrite_to_url(&None), Ok(None)));
    assert!(matches!(
      diesel_option_overwrite_to_url(&Some("".to_string())),
      Ok(Some(None))
    ));
    assert!(matches!(
      diesel_option_overwrite_to_url(&Some("invalid_url".to_string())),
      Err(_)
    ));
    let example_url = "https://example.com";
    assert!(matches!(
      diesel_option_overwrite_to_url(&Some(example_url.to_string())),
      Ok(Some(Some(url))) if url == Url::parse(example_url).unwrap().into()
    ));
  }
}
