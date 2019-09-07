use crate::Settings;
use diesel::dsl::*;
use diesel::result::Error;
use diesel::*;
use serde::{Deserialize, Serialize};

pub mod category;
pub mod comment;
pub mod comment_view;
pub mod community;
pub mod community_view;
pub mod moderator;
pub mod moderator_views;
pub mod post;
pub mod post_view;
pub mod user;
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
  fn ignore(conn: &PgConnection, form: &T) -> Result<usize, Error>
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

pub fn establish_connection() -> PgConnection {
  let db_url = Settings::get().db_url;
  PgConnection::establish(&db_url).expect(&format!("Error connecting to {}", db_url))
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
#[cfg(test)]
mod tests {
  use super::fuzzy_search;
  #[test]
  fn test_fuzzy_search() {
    let test = "This is a fuzzy search";
    assert_eq!(fuzzy_search(test), "%This%is%a%fuzzy%search%".to_string());
  }
}
