pub mod uplete;

use crate::{newtypes::DbUrl, schema_setup};
use chrono::TimeDelta;
use deadpool::Runtime;
use diesel::{
  dsl,
  expression::AsExpression,
  helper_types::AsExprOf,
  pg::{data_types::PgInterval, Pg},
  query_builder::{Query, QueryFragment},
  query_dsl::methods::LimitDsl,
  result::{
    ConnectionError,
    ConnectionResult,
    Error::{self as DieselError, QueryBuilderError},
  },
  sql_types::{self, Timestamptz},
  Expression,
  IntoSql,
};
use diesel_async::{
  pg::AsyncPgConnection,
  pooled_connection::{
    deadpool::{Hook, HookError, Object as PooledConnection, Pool},
    AsyncDieselConnectionManager,
    ManagerConfig,
  },
  AsyncConnection,
};
use futures_util::{future::BoxFuture, FutureExt};
use i_love_jesus::{CursorKey, PaginatedQueryBuilder};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::SETTINGS,
  utils::validation::clean_url,
};
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
  sync::{Arc, LazyLock, OnceLock},
  time::Duration,
};
use tracing::error;
use url::Url;

const FETCH_LIMIT_DEFAULT: i64 = 10;
pub const FETCH_LIMIT_MAX: i64 = 50;
pub const SITEMAP_LIMIT: i64 = 50000;
pub const SITEMAP_DAYS: TimeDelta = TimeDelta::days(31);
pub const RANK_DEFAULT: f64 = 0.0001;

/// Some connection options to speed up queries
const CONNECTION_OPTIONS: [&str; 1] = ["geqo_threshold=12"];
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

impl Deref for DbConn<'_> {
  type Target = AsyncPgConnection;

  fn deref(&self) -> &Self::Target {
    match self {
      DbConn::Pool(conn) => conn.deref(),
      DbConn::Conn(conn) => conn.deref(),
    }
  }
}

impl DerefMut for DbConn<'_> {
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
  type CursorValue = functions::reverse_timestamp_sort<K::CursorValue>;
  type SqlValue = functions::reverse_timestamp_sort<K::SqlValue>;

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

/// Takes an API optional text input, and converts it to an optional diesel DB update.
pub fn diesel_string_update(opt: Option<&str>) -> Option<Option<String>> {
  match opt {
    // An empty string is an erase
    Some("") => Some(None),
    Some(str) => Some(Some(str.into())),
    None => None,
  }
}

/// Takes an API optional number, and converts it to an optional diesel DB update. Zero means erase.
pub fn diesel_opt_number_update(opt: Option<i32>) -> Option<Option<i32>> {
  match opt {
    // Zero is an erase
    Some(0) => Some(None),
    Some(num) => Some(Some(num)),
    None => None,
  }
}

/// Takes an API optional text input, and converts it to an optional diesel DB update (for non
/// nullable properties).
pub fn diesel_required_string_update(opt: Option<&str>) -> Option<String> {
  match opt {
    // An empty string is no change
    Some("") => None,
    Some(str) => Some(str.into()),
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
      .map(|u| Some(Some(clean_url(&u).into())))
      .with_lemmy_type(LemmyErrorType::InvalidUrl),
    None => Ok(None),
  }
}

/// Takes an optional API URL-type input, and converts it to an optional diesel DB update (for non
/// nullable properties). Also cleans the url params.
pub fn diesel_required_url_update(opt: Option<&str>) -> LemmyResult<Option<DbUrl>> {
  match opt {
    // An empty string is no change
    Some("") => Ok(None),
    Some(str_url) => Url::parse(str_url)
      .map(|u| Some(clean_url(&u).into()))
      .with_lemmy_type(LemmyErrorType::InvalidUrl),
    None => Ok(None),
  }
}

/// Takes an optional API URL-type input, and converts it to an optional diesel DB create.
/// Also cleans the url params.
pub fn diesel_url_create(opt: Option<&str>) -> LemmyResult<Option<DbUrl>> {
  match opt {
    Some(str_url) => Url::parse(str_url)
      .map(|u| Some(clean_url(&u).into()))
      .with_lemmy_type(LemmyErrorType::InvalidUrl),
    None => Ok(None),
  }
}

/// Sets a few additional config options necessary for starting lemmy
fn build_config_options_uri_segment(config: &str) -> LemmyResult<String> {
  let mut url = Url::parse(config)?;

  // Set `lemmy.protocol_and_hostname` so triggers can use it
  let lemmy_protocol_and_hostname_option =
    "lemmy.protocol_and_hostname=".to_owned() + &SETTINGS.get_protocol_and_hostname();
  let mut options = CONNECTION_OPTIONS.to_vec();
  options.push(&lemmy_protocol_and_hostname_option);

  // Create the connection uri portion
  let options_segments = options
    .iter()
    .map(|o| "-c ".to_owned() + o)
    .collect::<Vec<String>>()
    .join(" ");

  url.set_query(Some(&format!("options={options_segments}")));
  Ok(url.into())
}

fn establish_connection(config: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
  let fut = async {
    /// Use a once_lock to create the postgres connection config, since this config never changes
    static POSTGRES_CONFIG_WITH_OPTIONS: OnceLock<String> = OnceLock::new();

    let config = POSTGRES_CONFIG_WITH_OPTIONS.get_or_init(|| {
      build_config_options_uri_segment(config)
        .inspect_err(|e| error!("Couldn't parse postgres connection URI: {e}"))
        .unwrap_or_default()
    });

    // We only support TLS with sslmode=require currently
    let conn = if config.contains("sslmode=require") {
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

pub fn build_db_pool() -> LemmyResult<ActualDbPool> {
  let db_url = SETTINGS.get_database_url();
  // diesel-async does not support any TLS connections out of the box, so we need to manually
  // provide a setup function which handles creating the connection
  let mut config = ManagerConfig::default();
  config.custom_setup = Box::new(establish_connection);
  let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(db_url, config);
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
        Err(HookError::Message("Connection is too old".into()))
      } else {
        Ok(())
      }
    }))
    .build()?;

  schema_setup::run(schema_setup::Options::default().run())?;

  Ok(pool)
}

#[allow(clippy::expect_used)]
pub fn build_db_pool_for_tests() -> ActualDbPool {
  build_db_pool().expect("db pool missing")
}

#[allow(clippy::expect_used)]
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$")
    .expect("compile email regex")
});

pub mod functions {
  use diesel::sql_types::{BigInt, Text, Timestamptz};

  define_sql_function! {
    #[sql_name = "r.hot_rank"]
    fn hot_rank(score: BigInt, time: Timestamptz) -> Double;
  }

  define_sql_function! {
    #[sql_name = "r.scaled_rank"]
    fn scaled_rank(score: BigInt, time: Timestamptz, interactions_month: BigInt) -> Double;
  }

  define_sql_function! {
    #[sql_name = "r.controversy_rank"]
    fn controversy_rank(upvotes: BigInt, downvotes: BigInt, score: BigInt) -> Double;
  }

  define_sql_function!(fn reverse_timestamp_sort(time: Timestamptz) -> BigInt);

  define_sql_function!(fn lower(x: Text) -> Text);

  define_sql_function!(fn random() -> Text);

  define_sql_function!(fn random_smallint() -> SmallInt);

  // really this function is variadic, this just adds the two-argument version
  define_sql_function!(fn coalesce<T: diesel::sql_types::SqlType + diesel::sql_types::SingleValue>(x: diesel::sql_types::Nullable<T>, y: T) -> T);

  define_sql_function! {
    #[aggregate]
    fn json_agg<T: diesel::sql_types::SqlType + diesel::sql_types::SingleValue>(obj: T) -> Json
  }

  define_sql_function!(#[sql_name = "coalesce"] fn coalesce_2_nullable<T: diesel::sql_types::SqlType + diesel::sql_types::SingleValue>(x: diesel::sql_types::Nullable<T>, y: diesel::sql_types::Nullable<T>) -> diesel::sql_types::Nullable<T>);
}

pub const DELETED_REPLACEMENT_TEXT: &str = "*Permanently Deleted*";

pub fn now() -> AsExprOf<diesel::dsl::now, diesel::sql_types::Timestamptz> {
  // https://github.com/diesel-rs/diesel/issues/1514
  diesel::dsl::now.into_sql::<Timestamptz>()
}

pub fn seconds_to_pg_interval(seconds: i32) -> PgInterval {
  PgInterval::from_microseconds(i64::from(seconds) * 1_000_000)
}

/// Trait alias for a type that can be converted to an SQL tuple using `IntoSql::into_sql`
pub trait AsRecord: Expression + AsExpression<sql_types::Record<Self::SqlType>>
where
  Self::SqlType: 'static,
{
}

impl<T: Expression + AsExpression<sql_types::Record<T::SqlType>>> AsRecord for T where
  T::SqlType: 'static
{
}

/// Output of `IntoSql::into_sql` for a type that implements `AsRecord`
pub type AsRecordOutput<T> = dsl::AsExprOf<T, sql_types::Record<<T as Expression>::SqlType>>;

pub type ResultFuture<'a, T> = BoxFuture<'a, Result<T, DieselError>>;

pub trait ReadFn<'a, T, Args>: Fn(DbConn<'a>, Args) -> ResultFuture<'a, T> {}

impl<'a, T, Args, F: Fn(DbConn<'a>, Args) -> ResultFuture<'a, T>> ReadFn<'a, T, Args> for F {}

pub trait ListFn<'a, T, Args>: Fn(DbConn<'a>, Args) -> ResultFuture<'a, Vec<T>> {}

impl<'a, T, Args, F: Fn(DbConn<'a>, Args) -> ResultFuture<'a, Vec<T>>> ListFn<'a, T, Args> for F {}

pub fn paginate<Q, C>(
  query: Q,
  page_after: Option<C>,
  page_before_or_equal: Option<C>,
  page_back: bool,
) -> PaginatedQueryBuilder<C, Q> {
  let mut query = PaginatedQueryBuilder::new(query);

  if page_back {
    query = query
      .before(page_after)
      .after_or_equal(page_before_or_equal)
      .limit_and_offset_from_end();
  } else {
    query = query
      .after(page_after)
      .before_or_equal(page_before_or_equal);
  }

  query
}

#[cfg(test)]
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
