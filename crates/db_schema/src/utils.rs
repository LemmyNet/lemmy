use crate::{diesel_migrations::MigrationHarness, newtypes::DbUrl, CommentSortType, SortType};
use activitypub_federation::{core::object_id::ObjectId, traits::ApubObject};
use chrono::NaiveDateTime;
use diesel::{
  backend::Backend,
  deserialize::FromSql,
  pg::Pg,
  result::Error::QueryBuilderError,
  serialize::{Output, ToSql},
  sql_types::Text,
  Connection,
  PgConnection,
};
use diesel_migrations::EmbeddedMigrations;
use lemmy_utils::error::LemmyError;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{env, env::VarError};
use url::Url;

const FETCH_LIMIT_DEFAULT: i64 = 10;
pub const FETCH_LIMIT_MAX: i64 = 50;

pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub fn get_database_url_from_env() -> Result<String, VarError> {
  env::var("LEMMY_DATABASE_URL")
}

pub fn fuzzy_search(q: &str) -> String {
  let replaced = q.replace('%', "\\%").replace('_', "\\_").replace(' ', "%");
  format!("%{}%", replaced)
}

pub fn limit_and_offset(
  page: Option<i64>,
  limit: Option<i64>,
) -> Result<(i64, i64), diesel::result::Error> {
  let page = match page {
    Some(page) => {
      if page < 1 {
        return Err(QueryBuilderError("Page is < 1".into()));
      } else {
        page
      }
    }
    None => 1,
  };
  let limit = match limit {
    Some(limit) => {
      if !(1..=FETCH_LIMIT_MAX).contains(&limit) {
        return Err(QueryBuilderError(
          format!("Fetch limit is > {}", FETCH_LIMIT_MAX).into(),
        ));
      } else {
        limit
      }
    }
    None => FETCH_LIMIT_DEFAULT,
  };
  let offset = limit * (page - 1);
  Ok((limit, offset))
}

pub fn limit_and_offset_unlimited(page: Option<i64>, limit: Option<i64>) -> (i64, i64) {
  let limit = limit.unwrap_or(FETCH_LIMIT_DEFAULT);
  let offset = limit * (page.unwrap_or(1) - 1);
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
) -> Result<Option<Option<DbUrl>>, LemmyError> {
  match opt.as_ref().map(|s| s.as_str()) {
    // An empty string is an erase
    Some("") => Ok(Some(None)),
    Some(str_url) => match Url::parse(str_url) {
      Ok(url) => Ok(Some(Some(url.into()))),
      Err(e) => Err(LemmyError::from_error_message(e, "invalid_url")),
    },
    None => Ok(None),
  }
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn establish_unpooled_connection() -> PgConnection {
  let db_url = match get_database_url_from_env() {
    Ok(url) => url,
    Err(e) => panic!(
      "Failed to read database URL from env var LEMMY_DATABASE_URL: {}",
      e
    ),
  };
  let mut conn =
    PgConnection::establish(&db_url).unwrap_or_else(|_| panic!("Error connecting to {}", db_url));
  let _ = &mut conn
    .run_pending_migrations(MIGRATIONS)
    .unwrap_or_else(|_| panic!("Couldn't run DB Migrations"));
  conn
}

pub fn naive_now() -> NaiveDateTime {
  chrono::prelude::Utc::now().naive_utc()
}

pub fn post_to_comment_sort_type(sort: SortType) -> CommentSortType {
  match sort {
    SortType::Active | SortType::Hot => CommentSortType::Hot,
    SortType::New | SortType::NewComments | SortType::MostComments => CommentSortType::New,
    SortType::Old => CommentSortType::Old,
    SortType::TopDay
    | SortType::TopAll
    | SortType::TopWeek
    | SortType::TopYear
    | SortType::TopMonth => CommentSortType::Top,
  }
}

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$")
    .expect("compile email regex")
});

pub mod functions {
  use diesel::sql_types::*;

  sql_function! {
    fn hot_rank(score: BigInt, time: Timestamp) -> Integer;
  }

  sql_function!(fn lower(x: Text) -> Text);
}

impl ToSql<Text, Pg> for DbUrl {
  fn to_sql(&self, out: &mut Output<Pg>) -> diesel::serialize::Result {
    <std::string::String as ToSql<Text, Pg>>::to_sql(&self.0.to_string(), &mut out.reborrow())
  }
}

impl<DB: Backend> FromSql<Text, DB> for DbUrl
where
  String: FromSql<Text, DB>,
{
  fn from_sql(value: diesel::backend::RawValue<'_, DB>) -> diesel::deserialize::Result<Self> {
    let str = String::from_sql(value)?;
    Ok(DbUrl(Url::parse(&str)?))
  }
}

impl<Kind> From<ObjectId<Kind>> for DbUrl
where
  Kind: ApubObject + Send + 'static,
  for<'de2> <Kind as ApubObject>::ApubType: serde::Deserialize<'de2>,
{
  fn from(id: ObjectId<Kind>) -> Self {
    DbUrl(id.into())
  }
}

#[cfg(test)]
mod tests {
  use super::{fuzzy_search, *};
  use crate::utils::is_email_regex;

  #[test]
  fn test_fuzzy_search() {
    let test = "This %is% _a_ fuzzy search";
    assert_eq!(
      fuzzy_search(test),
      "%This%\\%is\\%%\\_a\\_%fuzzy%search%".to_string()
    );
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
