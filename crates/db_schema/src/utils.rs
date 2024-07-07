use crate::{
  diesel::ExpressionMethods,
  newtypes::{DbUrl, PersonId},
  schema::community,
  CommentSortType,
  CommunityVisibility,
  SortType,
};
use chrono::{DateTime, TimeDelta, Utc};
use deadpool::Runtime;
use diesel::{
  dsl,
  helper_types::AsExprOf,
  pg::Pg,
  query_builder::{Query, QueryFragment},
  query_dsl::methods::LimitDsl,
  result::{
    ConnectionError,
    ConnectionResult,
    Error::{self as DieselError, QueryBuilderError},
  },
  sql_types::{self, Timestamptz},
  IntoSql,
  OptionalExtension,
};
use diesel_async::{
  pg::AsyncPgConnection,
  pooled_connection::{
    deadpool::{Hook, HookError, Object as PooledConnection, Pool},
    AsyncDieselConnectionManager,
    ManagerConfig,
  },
  AsyncConnection,
  RunQueryDsl,
};
use futures_util::{future::BoxFuture, Future, FutureExt};
use i_love_jesus::CursorKey;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::SETTINGS,
  utils::validation::clean_url_params,
};
use once_cell::sync::Lazy;
use regex::Regex;
use rustls::{
  client::danger::{
    DangerousClientConfigBuilder,
    HandshakeSignatureValid,
    ServerCertVerified,
    ServerCertVerifier,
  },
  crypto::{self, verify_tls12_signature, verify_tls13_signature},
  pki_types::{CertificateDer, ServerName, UnixTime},
  ClientConfig,
  DigitallySignedStruct,
  SignatureScheme,
};
use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
  time::Duration,
};
use tracing::error;
use url::Url;

const FETCH_LIMIT_DEFAULT: i64 = 10;
pub const FETCH_LIMIT_MAX: i64 = 50;
pub const SITEMAP_LIMIT: i64 = 50000;
pub const SITEMAP_DAYS: Option<TimeDelta> = TimeDelta::try_days(31);
pub const RANK_DEFAULT: f64 = 0.0001;

pub type ActualDbPool = Pool<AsyncPgConnection>;

/// References a pool or connection. Functions must take `&mut DbPool<'_>` to allow implicit
/// reborrowing.
///
/// https://github.com/rust-lang/rfcs/issues/1403
pub enum DbPool<'a> {
  Pool(&'a ActualDbPool),
  Conn(&'a mut AsyncPgConnection),
}

pub enum DbConn<'a> {
  Pool(PooledConnection<AsyncPgConnection>),
  Conn(&'a mut AsyncPgConnection),
}

pub async fn get_conn<'a, 'b: 'a>(pool: &'a mut DbPool<'b>) -> Result<DbConn<'a>, DieselError> {
  Ok(match pool {
    DbPool::Pool(pool) => DbConn::Pool(pool.get().await.map_err(|e| QueryBuilderError(e.into()))?),
    DbPool::Conn(conn) => DbConn::Conn(conn),
  })
}

impl<'a> Deref for DbConn<'a> {
  type Target = AsyncPgConnection;

  fn deref(&self) -> &Self::Target {
    match self {
      DbConn::Pool(conn) => conn.deref(),
      DbConn::Conn(conn) => conn.deref(),
    }
  }
}

impl<'a> DerefMut for DbConn<'a> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    match self {
      DbConn::Pool(conn) => conn.deref_mut(),
      DbConn::Conn(conn) => conn.deref_mut(),
    }
  }
}

// Allows functions that take `DbPool<'_>` to be called in a transaction by passing `&mut
// conn.into()`
impl<'a> From<&'a mut AsyncPgConnection> for DbPool<'a> {
  fn from(value: &'a mut AsyncPgConnection) -> Self {
    DbPool::Conn(value)
  }
}

impl<'a, 'b: 'a> From<&'a mut DbConn<'b>> for DbPool<'a> {
  fn from(value: &'a mut DbConn<'b>) -> Self {
    DbPool::Conn(value.deref_mut())
  }
}

impl<'a> From<&'a ActualDbPool> for DbPool<'a> {
  fn from(value: &'a ActualDbPool) -> Self {
    DbPool::Pool(value)
  }
}

/// Runs multiple async functions that take `&mut DbPool<'_>` as input and return `Result`. Only
/// works when the  `futures` crate is listed in `Cargo.toml`.
///
/// `$pool` is the value given to each function.
///
/// A `Result` is returned (not in a `Future`, so don't use `.await`). The `Ok` variant contains a
/// tuple with the values returned by the given functions.
///
/// The functions run concurrently if `$pool` has the `DbPool::Pool` variant.
#[macro_export]
macro_rules! try_join_with_pool {
  ($pool:ident => ($($func:expr),+)) => {{
    // Check type
    let _: &mut $crate::utils::DbPool<'_> = $pool;

    match $pool {
      // Run concurrently with `try_join`
      $crate::utils::DbPool::Pool(__pool) => ::futures::try_join!(
        $(async {
          let mut __dbpool = $crate::utils::DbPool::Pool(__pool);
          ($func)(&mut __dbpool).await
        }),+
      ),
      // Run sequentially
      $crate::utils::DbPool::Conn(__conn) => async {
        Ok(($({
          let mut __dbpool = $crate::utils::DbPool::Conn(__conn);
          // `?` prevents the error type from being inferred in an `async` block, so `match` is used instead
          match ($func)(&mut __dbpool).await {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__v) => return ::core::result::Result::Err(__v),
          }
        }),+))
      }.await,
    }
  }};
}

pub struct ReverseTimestampKey<K>(pub K);

impl<K, C> CursorKey<C> for ReverseTimestampKey<K>
where
  K: CursorKey<C, SqlType = Timestamptz>,
{
  type SqlType = sql_types::BigInt;
  type CursorValue = functions::reverse_timestamp_sort::HelperType<K::CursorValue>;
  type SqlValue = functions::reverse_timestamp_sort::HelperType<K::SqlValue>;

  fn get_cursor_value(cursor: &C) -> Self::CursorValue {
    functions::reverse_timestamp_sort(K::get_cursor_value(cursor))
  }

  fn get_sql_value() -> Self::SqlValue {
    functions::reverse_timestamp_sort(K::get_sql_value())
  }
}

/// Includes an SQL comment before `T`, which can be used to label auto_explain output
#[derive(QueryId)]
pub struct Commented<T> {
  comment: String,
  inner: T,
}

impl<T> Commented<T> {
  pub fn new(inner: T) -> Self {
    Commented {
      comment: String::new(),
      inner,
    }
  }

  /// Adds `text` to the comment if `condition` is true
  pub fn text_if(mut self, text: &str, condition: bool) -> Self {
    if condition {
      if !self.comment.is_empty() {
        self.comment.push_str(", ");
      }
      self.comment.push_str(text);
    }
    self
  }

  /// Adds `text` to the comment
  pub fn text(self, text: &str) -> Self {
    self.text_if(text, true)
  }
}

impl<T: Query> Query for Commented<T> {
  type SqlType = T::SqlType;
}

impl<T: QueryFragment<Pg>> QueryFragment<Pg> for Commented<T> {
  fn walk_ast<'b>(
    &'b self,
    mut out: diesel::query_builder::AstPass<'_, 'b, Pg>,
  ) -> Result<(), DieselError> {
    for line in self.comment.lines() {
      out.push_sql("\n-- ");
      out.push_sql(line);
    }
    out.push_sql("\n");
    self.inner.walk_ast(out.reborrow())
  }
}

impl<T: LimitDsl> LimitDsl for Commented<T> {
  type Output = Commented<T::Output>;

  fn limit(self, limit: i64) -> Self::Output {
    Commented {
      comment: self.comment,
      inner: self.inner.limit(limit),
    }
  }
}

pub fn fuzzy_search(q: &str) -> String {
  let replaced = q
    .replace('\\', "\\\\")
    .replace('%', "\\%")
    .replace('_', "\\_")
    .replace(' ', "%");
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
      }
      page
    }
    None => 1,
  };
  let limit = match limit {
    Some(limit) => {
      if !(1..=FETCH_LIMIT_MAX).contains(&limit) {
        return Err(QueryBuilderError(
          format!("Fetch limit is > {FETCH_LIMIT_MAX}").into(),
        ));
      }
      limit
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

/// Takes an API text input, and converts it to an optional diesel DB update.
pub fn diesel_string_update(opt: Option<&str>) -> Option<Option<String>> {
  match opt {
    // An empty string is an erase
    Some("") => Some(None),
    Some(str) => Some(Some(str.into())),
    None => None,
  }
}

/// Takes an optional API URL-type input, and converts it to an optional diesel DB update.
/// Also cleans the url params.
pub fn diesel_url_update(opt: Option<&str>) -> LemmyResult<Option<Option<DbUrl>>> {
  match opt {
    // An empty string is an erase
    Some("") => Ok(Some(None)),
    Some(str_url) => Url::parse(str_url)
      .map(|u| Some(Some(clean_url_params(&u).into())))
      .with_lemmy_type(LemmyErrorType::InvalidUrl),
    None => Ok(None),
  }
}

/// Takes an optional API URL-type input, and converts it to an optional diesel DB create.
/// Also cleans the url params.
pub fn diesel_url_create(opt: Option<&str>) -> LemmyResult<Option<DbUrl>> {
  match opt {
    Some(str_url) => Url::parse(str_url)
      .map(|u| Some(clean_url_params(&u).into()))
      .with_lemmy_type(LemmyErrorType::InvalidUrl),
    None => Ok(None),
  }
}

fn establish_connection(config: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
  let fut = async {
    // We only support TLS with sslmode=require currently
    let mut conn = if config.contains("sslmode=require") {
      rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

      let rustls_config = DangerousClientConfigBuilder {
        cfg: ClientConfig::builder(),
      }
      .with_custom_certificate_verifier(Arc::new(NoCertVerifier {}))
      .with_no_client_auth();

      let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);
      let (client, conn) = tokio_postgres::connect(config, tls)
        .await
        .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;
      tokio::spawn(async move {
        if let Err(e) = conn.await {
          error!("Database connection failed: {e}");
        }
      });
      AsyncPgConnection::try_from(client).await?
    } else {
      AsyncPgConnection::establish(config).await?
    };

    diesel::select((
      // Change geqo_threshold back to default value if it was changed, so it's higher than the
      // collapse limits
      functions::set_config("geqo_threshold", "12", false),
      // Change collapse limits from 8 to 11 so the query planner can find a better table join
      // order for more complicated queries
      functions::set_config("from_collapse_limit", "11", false),
      functions::set_config("join_collapse_limit", "11", false),
      // Set `lemmy.protocol_and_hostname` so triggers can use it
      functions::set_config(
        "lemmy.protocol_and_hostname",
        SETTINGS.get_protocol_and_hostname(),
        false,
      ),
    ))
    .execute(&mut conn)
    .await
    .map_err(ConnectionError::CouldntSetupConfiguration)?;
    Ok(conn)
  };
  fut.boxed()
}

#[derive(Debug)]
struct NoCertVerifier {}

impl ServerCertVerifier for NoCertVerifier {
  fn verify_server_cert(
    &self,
    _end_entity: &CertificateDer,
    _intermediates: &[CertificateDer],
    _server_name: &ServerName,
    _ocsp: &[u8],
    _now: UnixTime,
  ) -> Result<ServerCertVerified, rustls::Error> {
    // Will verify all (even invalid) certs without any checks (sslmode=require)
    Ok(ServerCertVerified::assertion())
  }

  fn verify_tls12_signature(
    &self,
    message: &[u8],
    cert: &CertificateDer,
    dss: &DigitallySignedStruct,
  ) -> Result<HandshakeSignatureValid, rustls::Error> {
    verify_tls12_signature(
      message,
      cert,
      dss,
      &crypto::ring::default_provider().signature_verification_algorithms,
    )
  }

  fn verify_tls13_signature(
    &self,
    message: &[u8],
    cert: &CertificateDer,
    dss: &DigitallySignedStruct,
  ) -> Result<HandshakeSignatureValid, rustls::Error> {
    verify_tls13_signature(
      message,
      cert,
      dss,
      &crypto::ring::default_provider().signature_verification_algorithms,
    )
  }

  fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
    crypto::ring::default_provider()
      .signature_verification_algorithms
      .supported_schemes()
  }
}

pub async fn build_db_pool() -> LemmyResult<ActualDbPool> {
  let db_url = SETTINGS.get_database_url();
  // diesel-async does not support any TLS connections out of the box, so we need to manually
  // provide a setup function which handles creating the connection
  let mut config = ManagerConfig::default();
  config.custom_setup = Box::new(establish_connection);
  let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(&db_url, config);
  let pool = Pool::builder(manager)
    .max_size(SETTINGS.database.pool_size)
    .runtime(Runtime::Tokio1)
    // Limit connection age to prevent use of prepared statements that have query plans based on
    // very old statistics
    .pre_recycle(Hook::sync_fn(|_conn, metrics| {
      // Preventing the first recycle can cause an infinite loop when trying to get a new connection
      // from the pool
      let conn_was_used = metrics.recycled.is_some();
      if metrics.age() > Duration::from_secs(3 * 24 * 60 * 60) && conn_was_used {
        Err(HookError::Continue(None))
      } else {
        Ok(())
      }
    }))
    .build()?;

  crate::schema_setup::run(&db_url)?;

  Ok(pool)
}

pub async fn build_db_pool_for_tests() -> ActualDbPool {
  build_db_pool().await.expect("db pool missing")
}

pub fn naive_now() -> DateTime<Utc> {
  Utc::now()
}

pub fn post_to_comment_sort_type(sort: SortType) -> CommentSortType {
  match sort {
    SortType::Active | SortType::Hot | SortType::Scaled => CommentSortType::Hot,
    SortType::New | SortType::NewComments | SortType::MostComments => CommentSortType::New,
    SortType::Old => CommentSortType::Old,
    SortType::Controversial => CommentSortType::Controversial,
    SortType::TopHour
    | SortType::TopSixHour
    | SortType::TopTwelveHour
    | SortType::TopDay
    | SortType::TopAll
    | SortType::TopWeek
    | SortType::TopYear
    | SortType::TopMonth
    | SortType::TopThreeMonths
    | SortType::TopSixMonths
    | SortType::TopNineMonths => CommentSortType::Top,
  }
}

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$")
    .expect("compile email regex")
});

pub mod functions {
  use diesel::sql_types::{BigInt, Bool, Text, Timestamptz};

  sql_function! {
    #[sql_name = "r.hot_rank"]
    fn hot_rank(score: BigInt, time: Timestamptz) -> Double;
  }

  sql_function! {
    #[sql_name = "r.scaled_rank"]
    fn scaled_rank(score: BigInt, time: Timestamptz, users_active_month: BigInt) -> Double;
  }

  sql_function! {
    #[sql_name = "r.controversy_rank"]
    fn controversy_rank(upvotes: BigInt, downvotes: BigInt, score: BigInt) -> Double;
  }

  sql_function!(fn reverse_timestamp_sort(time: Timestamptz) -> BigInt);

  sql_function!(fn lower(x: Text) -> Text);

  // really this function is variadic, this just adds the two-argument version
  sql_function!(fn coalesce<T: diesel::sql_types::SqlType + diesel::sql_types::SingleValue>(x: diesel::sql_types::Nullable<T>, y: T) -> T);

  sql_function!(fn set_config(setting_name: Text, new_value: Text, is_local: Bool) -> Text);
}

pub const DELETED_REPLACEMENT_TEXT: &str = "*Permanently Deleted*";

pub fn now() -> AsExprOf<diesel::dsl::now, diesel::sql_types::Timestamptz> {
  // https://github.com/diesel-rs/diesel/issues/1514
  diesel::dsl::now.into_sql::<Timestamptz>()
}

pub type ResultFuture<'a, T> = BoxFuture<'a, Result<T, DieselError>>;

pub trait ReadFn<'a, T, Args>: Fn(DbConn<'a>, Args) -> ResultFuture<'a, T> {}

impl<'a, T, Args, F: Fn(DbConn<'a>, Args) -> ResultFuture<'a, T>> ReadFn<'a, T, Args> for F {}

pub trait ListFn<'a, T, Args>: Fn(DbConn<'a>, Args) -> ResultFuture<'a, Vec<T>> {}

impl<'a, T, Args, F: Fn(DbConn<'a>, Args) -> ResultFuture<'a, Vec<T>>> ListFn<'a, T, Args> for F {}

/// Allows read and list functions to capture a shared closure that has an inferred return type,
/// which is useful for join logic
pub struct Queries<RF, LF> {
  pub read_fn: RF,
  pub list_fn: LF,
}

// `()` is used to prevent type inference error
impl Queries<(), ()> {
  pub fn new<'a, RFut, LFut, RT, LT, RA, LA, RF2, LF2>(
    read_fn: RF2,
    list_fn: LF2,
  ) -> Queries<impl ReadFn<'a, RT, RA>, impl ListFn<'a, LT, LA>>
  where
    RFut: Future<Output = Result<RT, DieselError>> + Sized + Send + 'a,
    LFut: Future<Output = Result<Vec<LT>, DieselError>> + Sized + Send + 'a,
    RF2: Fn(DbConn<'a>, RA) -> RFut,
    LF2: Fn(DbConn<'a>, LA) -> LFut,
  {
    Queries {
      read_fn: move |conn, args| read_fn(conn, args).boxed(),
      list_fn: move |conn, args| list_fn(conn, args).boxed(),
    }
  }
}

impl<RF, LF> Queries<RF, LF> {
  pub async fn read<'a, T, Args>(
    self,
    pool: &'a mut DbPool<'_>,
    args: Args,
  ) -> Result<Option<T>, DieselError>
  where
    RF: ReadFn<'a, T, Args>,
  {
    let conn = get_conn(pool).await?;
    (self.read_fn)(conn, args).await.optional()
  }

  pub async fn list<'a, T, Args>(
    self,
    pool: &'a mut DbPool<'_>,
    args: Args,
  ) -> Result<Vec<T>, DieselError>
  where
    LF: ListFn<'a, T, Args>,
  {
    let conn = get_conn(pool).await?;
    (self.list_fn)(conn, args).await
  }
}

pub fn visible_communities_only<Q>(my_person_id: Option<PersonId>, query: Q) -> Q
where
  Q: diesel::query_dsl::methods::FilterDsl<
    dsl::Eq<community::visibility, CommunityVisibility>,
    Output = Q,
  >,
{
  if my_person_id.is_none() {
    query.filter(community::visibility.eq(CommunityVisibility::Public))
  } else {
    query
  }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use super::*;
  use pretty_assertions::assert_eq;

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
    assert_eq!(diesel_string_update(None), None);
    assert_eq!(diesel_string_update(Some("")), Some(None));
    assert_eq!(
      diesel_string_update(Some("test")),
      Some(Some("test".to_string()))
    );
  }

  #[test]
  fn test_diesel_option_overwrite_to_url() -> LemmyResult<()> {
    assert!(matches!(diesel_url_update(None), Ok(None)));
    assert!(matches!(diesel_url_update(Some("")), Ok(Some(None))));
    assert!(diesel_url_update(Some("invalid_url")).is_err());
    let example_url = "https://example.com";
    assert!(matches!(
      diesel_url_update(Some(example_url)),
      Ok(Some(Some(url))) if url == Url::parse(example_url)?.into()
    ));
    Ok(())
  }
}
