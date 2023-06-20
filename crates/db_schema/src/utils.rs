use crate::{
  diesel::Connection,
  diesel_migrations::MigrationHarness,
  newtypes::DbUrl,
  CommentSortType,
  SortType,
};
use activitypub_federation::{fetch::object_id::ObjectId, traits::Object};
use chrono::NaiveDateTime;
use deadpool::Runtime;
use diesel::{
  backend::Backend,
  deserialize::FromSql,
  pg::Pg,
  result::{Error as DieselError, Error::QueryBuilderError},
  serialize::{Output, ToSql},
  sql_types::Text,
  PgConnection,
};
use diesel_async::{
  pg::AsyncPgConnection,
  pooled_connection::{
    deadpool::{Object as PooledConnection, Pool},
    AsyncDieselConnectionManager,
  },
};
use diesel_migrations::EmbeddedMigrations;
use lemmy_utils::{error::LemmyError, settings::structs::Settings};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{env, env::VarError, time::Duration};
use tracing::info;
use url::Url;

const FETCH_LIMIT_DEFAULT: i64 = 10;
pub const FETCH_LIMIT_MAX: i64 = 50;
const POOL_TIMEOUT: Option<Duration> = Some(Duration::from_secs(5));

pub type DbPool = Pool<AsyncPgConnection>;

pub async fn get_conn(pool: &DbPool) -> Result<PooledConnection<AsyncPgConnection>, DieselError> {
  pool.get().await.map_err(|e| QueryBuilderError(e.into()))
}

pub fn get_database_url_from_env() -> Result<String, VarError> {
  env::var("LEMMY_DATABASE_URL")
}

pub fn fuzzy_search(q: &str) -> String {
  let replaced = q.replace('%', "\\%").replace('_', "\\_").replace(' ', "%");
  format!("%{replaced}%")
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
          format!("Fetch limit is > {FETCH_LIMIT_MAX}").into(),
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
        Some(Some(unwrapped.clone()))
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
  match opt.as_ref().map(std::string::String::as_str) {
    // An empty string is an erase
    Some("") => Ok(Some(None)),
    Some(str_url) => match Url::parse(str_url) {
      Ok(url) => Ok(Some(Some(url.into()))),
      Err(e) => Err(LemmyError::from_error_message(e, "invalid_url")),
    },
    None => Ok(None),
  }
}

pub fn diesel_option_overwrite_to_url_create(
  opt: &Option<String>,
) -> Result<Option<DbUrl>, LemmyError> {
  match opt.as_ref().map(std::string::String::as_str) {
    // An empty string is nothing
    Some("") => Ok(None),
    Some(str_url) => match Url::parse(str_url) {
      Ok(url) => Ok(Some(url.into())),
      Err(e) => Err(LemmyError::from_error_message(e, "invalid_url")),
    },
    None => Ok(None),
  }
}

async fn build_db_pool_settings_opt(settings: Option<&Settings>) -> Result<DbPool, LemmyError> {
  let db_url = get_database_url(settings);
  let pool_size = settings.map(|s| s.database.pool_size).unwrap_or(5);
  let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&db_url);
  let pool = Pool::builder(manager)
    .max_size(pool_size)
    .wait_timeout(POOL_TIMEOUT)
    .create_timeout(POOL_TIMEOUT)
    .recycle_timeout(POOL_TIMEOUT)
    .runtime(Runtime::Tokio1)
    .build()?;

  // If there's no settings, that means its a unit test, and migrations need to be run
  if settings.is_none() {
    run_migrations(&db_url);
  }

  Ok(pool)
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn run_migrations(db_url: &str) {
  // Needs to be a sync connection
  let mut conn =
    PgConnection::establish(db_url).unwrap_or_else(|e| panic!("Error connecting to {db_url}: {e}"));
  info!("Running Database migrations (This may take a long time)...");
  let _ = &mut conn
    .run_pending_migrations(MIGRATIONS)
    .unwrap_or_else(|e| panic!("Couldn't run DB Migrations: {e}"));
  info!("Database migrations complete.");
}

pub async fn build_db_pool(settings: &Settings) -> Result<DbPool, LemmyError> {
  build_db_pool_settings_opt(Some(settings)).await
}

pub async fn build_db_pool_for_tests() -> DbPool {
  build_db_pool_settings_opt(None)
    .await
    .expect("db pool missing")
}

pub fn get_database_url(settings: Option<&Settings>) -> String {
  // The env var should override anything in the settings config
  match get_database_url_from_env() {
    Ok(url) => url,
    Err(e) => match settings {
      Some(settings) => settings.get_database_url(),
      None => panic!("Failed to read database URL from env var LEMMY_DATABASE_URL: {e}"),
    },
  }
}

pub fn naive_now() -> NaiveDateTime {
  chrono::prelude::Utc::now().naive_utc()
}

pub fn post_to_comment_sort_type(sort: SortType) -> CommentSortType {
  match sort {
    SortType::Active | SortType::Hot => CommentSortType::Hot,
    SortType::New | SortType::NewComments | SortType::MostComments => CommentSortType::New,
    SortType::Old => CommentSortType::Old,
    SortType::TopHour
    | SortType::TopSixHour
    | SortType::TopTwelveHour
    | SortType::TopDay
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
  use diesel::sql_types::{BigInt, Text, Timestamp};

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
  fn from_sql(value: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
    let str = String::from_sql(value)?;
    Ok(DbUrl(Box::new(Url::parse(&str)?)))
  }
}

impl<Kind> From<ObjectId<Kind>> for DbUrl
where
  Kind: Object + Send + 'static,
  for<'de2> <Kind as Object>::Kind: serde::Deserialize<'de2>,
{
  fn from(id: ObjectId<Kind>) -> Self {
    DbUrl(Box::new(id.into()))
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
    assert_eq!(diesel_option_overwrite(&Some(String::new())), Some(None));
    assert_eq!(
      diesel_option_overwrite(&Some("test".to_string())),
      Some(Some("test".to_string()))
    );
  }

  #[test]
  fn test_diesel_option_overwrite_to_url() {
    assert!(matches!(diesel_option_overwrite_to_url(&None), Ok(None)));
    assert!(matches!(
      diesel_option_overwrite_to_url(&Some(String::new())),
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
