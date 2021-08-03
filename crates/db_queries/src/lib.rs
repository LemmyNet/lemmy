#[macro_use]
extern crate diesel;
#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate lazy_static;
// this is used in tests
#[allow(unused_imports)]
#[macro_use]
extern crate diesel_migrations;

#[cfg(test)]
extern crate serial_test;

use diesel::{*, r2d2::{ConnectionManager, Pool}, result::Error};
use lemmy_db_schema::{CommunityId, DbUrl, PersonId};
use lemmy_utils::{ApiError, settings::structs::Settings};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{env, env::VarError};
use url::Url;
use core::pin::Pin;
use core::future::Future;
use tokio_diesel::*;

pub mod aggregates;
pub mod source;

pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;
pub type TokioDieselFuture<'a, T>= Pin<Box<dyn Future<Output = Result<T, AsyncError>> + Send + 'a>>;

pub trait Crud<'a, Form, IdType> {
  fn create(pool: &'a DbPool, form: &'a Form) -> TokioDieselFuture<'a, Self>
  where
    Self: Sized;
  fn read(pool: &'a DbPool, id: IdType) -> TokioDieselFuture<'a, Self>
  where
    Self: Sized;
  fn update(pool: &'a DbPool, id: IdType, form: &'a Form) -> TokioDieselFuture<'a, Self>  
  where
    Self: Sized;
  fn delete(_pool: &'a DbPool, _id: IdType) -> TokioDieselFuture<'a, usize>
  where
    Self: Sized,
  {
    unimplemented!()
  }
}

pub trait Followable<'a, Form> {
  fn follow(pool: &'a DbPool, form: &Form) -> TokioDieselFuture<'a, Self>
  where
    Self: Sized;
  fn follow_accepted(
    pool: &'a DbPool,
    community_id: CommunityId,
    person_id: PersonId,
  ) -> TokioDieselFuture<'a, Self>
  where
    Self: Sized;
  fn unfollow(pool: &'a DbPool, form: &Form) -> TokioDieselFuture<'a, usize>
  where
    Self: Sized;
  fn has_local_followers(pool: &'a DbPool, community_id: CommunityId) -> Result<bool, Error>;
}

pub trait Joinable<'a, Form> {
  fn join(pool: &'a DbPool, form: &Form) -> TokioDieselFuture<'a, Self>
  where
    Self: Sized;
  fn leave(pool: &'a DbPool, form: &Form) -> TokioDieselFuture<'a, usize>
  where
    Self: Sized;
}

pub trait Likeable<'a, Form, IdType> {
  fn like(pool: &'a DbPool, form: &'a Form) -> TokioDieselFuture<'a, Self>
  where
    Self: Sized;
  fn remove(pool: &'a DbPool, person_id: PersonId, item_id: IdType) -> TokioDieselFuture<'a, usize>
  where
    Self: Sized;
}

pub trait Bannable<'a, Form> {
  fn ban(pool: &'a DbPool, form: &'a Form) -> TokioDieselFuture<'a, Self>
  where
    Self: Sized;
  fn unban(pool: &'a DbPool, form: &'a Form) -> TokioDieselFuture<'a, usize>
  where
    Self: Sized;
}

pub trait Saveable<'a, Form> {
  fn save(pool: &'a DbPool, form: &'a Form) -> TokioDieselFuture<'a, Self>
  where
    Self: Sized;
  fn unsave(pool: &'a DbPool, form: &'a Form) -> TokioDieselFuture<'a, usize>
  where
    Self: Sized;
}

pub trait Readable<'a, Form> {
  fn mark_as_read(pool: &'a DbPool, form: &'a Form) -> TokioDieselFuture<'a, Self>
  where
    Self: Sized;
  fn mark_as_unread(pool: &'a DbPool, form: &'a Form) -> TokioDieselFuture<'a, usize>
  where
    Self: Sized;
}

pub trait Reportable<'a, Form> {
  fn report(pool: &'a DbPool, form: &'a Form) -> TokioDieselFuture<'a, Self>
  where
    Self: Sized;
  fn resolve(pool: &'a DbPool, report_id: i32, resolver_id: PersonId) -> TokioDieselFuture<'a, usize>
  where
    Self: Sized;
  fn unresolve(pool: &'a DbPool, report_id: i32, resolver_id: PersonId) -> TokioDieselFuture<'a, usize>
  where
    Self: Sized;
}

pub trait DeleteableOrRemoveable {
  fn blank_out_deleted_or_removed_info(self) -> Self;
}

pub trait ApubObject<'a, Form> {
  fn read_from_apub_id(pool: &'a DbPool, object_id: &'a DbUrl) -> TokioDieselFuture<'a, Self>
  where
    Self: Sized;
  fn upsert(pool: &'a DbPool, user_form: &'a Form) -> TokioDieselFuture<'a, Self>
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

pub trait ToSafe {
  type SafeColumns;
  fn safe_columns_tuple() -> Self::SafeColumns;
}

pub trait ToSafeSettings {
  type SafeSettingsColumns;
  fn safe_settings_columns_tuple() -> Self::SafeSettingsColumns;
}

pub trait ViewToVec {
  type DbTuple;
  fn from_tuple_to_vec(tuple: Vec<Self::DbTuple>) -> Vec<Self>
  where
    Self: Sized;
}

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
      Err(_) => Err(ApiError::err("invalid_url")),
    },
    None => Ok(None),
  }
}

embed_migrations!();

/// Set up the r2d2 connection pool
pub fn setup_connection_pool() -> DbPool {
  let db_url = match get_database_url_from_env() {
    Ok(url) => url,
    Err(_) => Settings::get().get_database_url(),
  };
  build_connection_pool(&db_url, Settings::get().database().pool_size())
}

/// Set up the r2d2 connection pool for tests
pub fn setup_connection_pool_for_tests() -> DbPool {
  let db_url = match get_database_url_from_env() {
    Ok(url) => url,
    Err(e) => panic!(
      "Failed to read database URL from env var LEMMY_DATABASE_URL: {}",
      e
    ),
  };
  build_connection_pool(&db_url, 10)
}

fn build_connection_pool(db_url: &str, pool_size: u32) -> DbPool {
  let manager = ConnectionManager::<PgConnection>::new(db_url);
  let pool = Pool::builder()
    .max_size(pool_size)
    .build(manager)
    .unwrap_or_else(|_| panic!("Error connecting to {}", db_url));
  let conn = pool.get().expect("Missing connection in pool");

  // Run the migrations
  embedded_migrations::run(&conn).expect("load migrations");

  pool
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
