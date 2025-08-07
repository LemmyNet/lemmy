pub mod queries;
pub mod queryable;

use crate::newtypes::DbUrl;
use chrono::TimeDelta;
use deadpool::Runtime;
use diesel::{
  dsl,
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
  scoped_futures::ScopedBoxFuture,
  AsyncConnection,
};
use futures_util::{future::BoxFuture, FutureExt};
use i_love_jesus::{CursorKey, PaginatedQueryBuilder, SortDirection};
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::{structs::Settings, SETTINGS},
  utils::validation::clean_url,
};
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

const FETCH_LIMIT_DEFAULT: i64 = 20;
pub const FETCH_LIMIT_MAX: usize = 50;
pub const SITEMAP_LIMIT: i64 = 50000;
pub const SITEMAP_DAYS: TimeDelta = TimeDelta::days(31);
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

impl DbConn<'_> {
  pub async fn run_transaction<'a, R, F>(&mut self, callback: F) -> LemmyResult<R>
  where
    F: for<'r> FnOnce(&'r mut AsyncPgConnection) -> ScopedBoxFuture<'a, 'r, LemmyResult<R>>
      + Send
      + 'a,
    R: Send + 'a,
  {
    self
      .deref_mut()
      .transaction::<_, LemmyError, _>(callback)
      .await
  }
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
      $crate::utils::DbPool::Pool(__pool) => ::futures_util::try_join!(
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

/// Necessary to be able to use cursors with the lower SQL function
pub struct LowerKey<K>(pub K);

impl<K, C> CursorKey<C> for LowerKey<K>
where
  K: CursorKey<C, SqlType = sql_types::Text>,
{
  type SqlType = sql_types::Text;
  type CursorValue = functions::lower<K::CursorValue>;
  type SqlValue = functions::lower<K::SqlValue>;

  fn get_cursor_value(cursor: &C) -> Self::CursorValue {
    functions::lower(K::get_cursor_value(cursor))
  }

  fn get_sql_value() -> Self::SqlValue {
    functions::lower(K::get_sql_value())
  }
}

/// Necessary to be able to use cursors with the subpath SQL function
pub struct Subpath<K>(pub K);

impl<K, C> CursorKey<C> for Subpath<K>
where
  K: CursorKey<C, SqlType = diesel_ltree::sql_types::Ltree>,
{
  type SqlType = diesel_ltree::sql_types::Ltree;
  type CursorValue = diesel_ltree::subpath<K::CursorValue, i32, i32>;
  type SqlValue = diesel_ltree::subpath<K::SqlValue, i32, i32>;

  fn get_cursor_value(cursor: &C) -> Self::CursorValue {
    diesel_ltree::subpath(K::get_cursor_value(cursor), 0, -1)
  }

  fn get_sql_value() -> Self::SqlValue {
    diesel_ltree::subpath(K::get_sql_value(), 0, -1)
  }
}

pub struct ScoreKey<U, D> {
  pub non_1_upvotes: U,
  pub non_0_downvotes: D,
}

impl<U, D, C> CursorKey<C> for ScoreKey<U, D>
where
  U: CursorKey<C, SqlType = sql_types::Nullable<sql_types::Integer>>,
  D: CursorKey<C, SqlType = sql_types::Nullable<sql_types::Integer>>,
{
  type SqlType = sql_types::Integer;
  //type CursorValue = dsl::AsExprOf<i32, sql_types::Integer>;
  type CursorValue = functions::get_score<U::CursorValue, D::CursorValue>;
  type SqlValue = functions::get_score<U::SqlValue, D::SqlValue>;

  fn get_cursor_value(cursor: &C) -> Self::CursorValue {
    /* TODO: use this when the CursorKey trait is changed to allow non-binded CursorValue
    let upvotes = U::get_cursor_value(cursor).unwrap_or(1);
    let downvotes = D::get_cursor_value(cursor).unwrap_or(0);
    upvotes.saturating_sub(downvotes).into_sql::<sql_types::Integer>()*/
    functions::get_score(U::get_cursor_value(cursor), D::get_cursor_value(cursor))
  }

  fn get_sql_value() -> Self::SqlValue {
    functions::get_score(U::get_sql_value(), D::get_sql_value())
  }
}

pub struct ControversyRankKey<U, D> {
  pub non_1_upvotes: U,
  pub non_0_downvotes: D,
}

impl<U, D, C> CursorKey<C> for ControversyRankKey<U, D>
where
  U: CursorKey<C, SqlType = sql_types::Nullable<sql_types::Integer>>,
  D: CursorKey<C, SqlType = sql_types::Nullable<sql_types::Integer>>,
{
  type SqlType = sql_types::Float;
  type CursorValue = functions::get_controversy_rank<U::CursorValue, D::CursorValue>;
  type SqlValue = functions::get_controversy_rank<U::SqlValue, D::SqlValue>;

  fn get_cursor_value(cursor: &C) -> Self::CursorValue {
    // make the postgresql server evaluate the expression, because implementations of floating point
    // operations vary
    functions::get_controversy_rank(U::get_cursor_value(cursor), D::get_cursor_value(cursor))
  }

  fn get_sql_value() -> Self::SqlValue {
    functions::get_controversy_rank(U::get_sql_value(), D::get_sql_value())
  }
}

pub struct HotRankKey<U, D, A> {
  pub non_1_upvotes: U,
  pub non_0_downvotes: D,
  pub age: A,
}

impl<U, D, A, C> CursorKey<C> for HotRankKey<U, D, A>
where
  U: CursorKey<C, SqlType = sql_types::Nullable<sql_types::Integer>>,
  D: CursorKey<C, SqlType = sql_types::Nullable<sql_types::Integer>>,
  A: CursorKey<C, SqlType = sql_types::Nullable<sql_types::SmallInt>>,
{
  type SqlType = sql_types::Float;
  type CursorValue = functions::get_hot_rank<U::CursorValue, D::CursorValue, A::CursorValue>;
  type SqlValue = functions::get_hot_rank<U::SqlValue, D::SqlValue, A::SqlValue>;

  fn get_cursor_value(cursor: &C) -> Self::CursorValue {
    // make the postgresql server evaluate the expression, because implementations of floating point
    // operations vary
    functions::get_hot_rank(
      U::get_cursor_value(cursor),
      D::get_cursor_value(cursor),
      A::get_cursor_value(cursor),
    )
  }

  fn get_sql_value() -> Self::SqlValue {
    functions::get_hot_rank(U::get_sql_value(), D::get_sql_value(), A::get_sql_value())
  }
}

pub struct ScaledRankKey<U, D, A, I> {
  pub non_1_upvotes: U,
  pub non_0_downvotes: D,
  pub age: A,
  pub non_0_community_interactions_month: I,
}

impl<U, D, A, I, C> CursorKey<C> for ScaledRankKey<U, D, A, I>
where
  U: CursorKey<C, SqlType = sql_types::Nullable<sql_types::Integer>>,
  D: CursorKey<C, SqlType = sql_types::Nullable<sql_types::Integer>>,
  A: CursorKey<C, SqlType = sql_types::Nullable<sql_types::SmallInt>>,
  I: CursorKey<C, SqlType = sql_types::Nullable<sql_types::Integer>>,
{
  type SqlType = sql_types::Float;
  type CursorValue =
    functions::get_scaled_rank<U::CursorValue, D::CursorValue, A::CursorValue, I::CursorValue>;
  type SqlValue = functions::get_scaled_rank<U::SqlValue, D::SqlValue, A::SqlValue, I::SqlValue>;

  fn get_cursor_value(cursor: &C) -> Self::CursorValue {
    // make the postgresql server evaluate the expression, because implementations of floating point
    // operations vary
    functions::get_scaled_rank(
      U::get_cursor_value(cursor),
      D::get_cursor_value(cursor),
      A::get_cursor_value(cursor),
      I::get_cursor_value(cursor),
    )
  }

  fn get_sql_value() -> Self::SqlValue {
    functions::get_scaled_rank(
      U::get_sql_value(),
      D::get_sql_value(),
      A::get_sql_value(),
      I::get_sql_value(),
    )
  }
}

pub struct CommunityHotRankKey<U, A> {
  pub non_1_subscribers: U,
  pub age: A,
}

impl<U, A, C> CursorKey<C> for CommunityHotRankKey<U, A>
where
  U: CursorKey<C, SqlType = sql_types::Nullable<sql_types::Integer>>,
  A: CursorKey<C, SqlType = sql_types::Nullable<sql_types::SmallInt>>,
{
  type SqlType = sql_types::Float;
  type CursorValue = functions::get_community_hot_rank<U::CursorValue, A::CursorValue>;
  type SqlValue = functions::get_community_hot_rank<U::SqlValue, A::SqlValue>;

  fn get_cursor_value(cursor: &C) -> Self::CursorValue {
    // make the postgresql server evaluate the expression, because implementations of floating point
    // operations vary
    functions::get_community_hot_rank(U::get_cursor_value(cursor), A::get_cursor_value(cursor))
  }

  fn get_sql_value() -> Self::SqlValue {
    functions::get_community_hot_rank(U::get_sql_value(), A::get_sql_value())
  }
}

pub struct Coalesce2NullableKey<A, B>(pub A, pub B);

// TODO: make it generic, not just for smallint/i16
impl<A, B, C> CursorKey<C> for Coalesce2NullableKey<A, B>
where
  A: CursorKey<C, SqlType = sql_types::Nullable<sql_types::SmallInt>>,
  B: CursorKey<C, SqlType = sql_types::Nullable<sql_types::SmallInt>>,
{
  type SqlType = sql_types::Nullable<sql_types::SmallInt>;
  //type CursorValue = AsExprOf<Option<i16>, sql_types::Nullable<sql_types::SmallInt>>;
  type CursorValue =
    functions::coalesce_2_nullable<sql_types::SmallInt, A::CursorValue, B::CursorValue>;
  type SqlValue = functions::coalesce_2_nullable<sql_types::SmallInt, A::SqlValue, B::SqlValue>;

  fn get_cursor_value(cursor: &C) -> Self::CursorValue {
    /* TODO: use this when the CursorKey trait is changed to allow non-binded CursorValue
    A::get_cursor_value(cursor).or_else(|| B::get_cursor_value(cursor)).into_sql::<sql_types::Nullable<sql_types::SmallInt>>()*/
    functions::coalesce_2_nullable(A::get_cursor_value(cursor), B::get_cursor_value(cursor))
  }

  fn get_sql_value() -> Self::SqlValue {
    functions::coalesce_2_nullable(A::get_sql_value(), B::get_sql_value())
  }
}

pub struct CoalesceKey<A, B>(pub A, pub B);

// TODO: make it generic, not just for time
impl<A, B, C> CursorKey<C> for CoalesceKey<A, B>
where
  A: CursorKey<C, SqlType = sql_types::Nullable<sql_types::Timestamptz>>,
  B: CursorKey<C, SqlType = sql_types::Timestamptz>,
{
  type SqlType = sql_types::Timestamptz;
  type CursorValue = functions::coalesce<sql_types::Timestamptz, A::CursorValue, B::CursorValue>;
  type SqlValue = functions::coalesce<sql_types::Timestamptz, A::SqlValue, B::SqlValue>;

  fn get_cursor_value(cursor: &C) -> Self::CursorValue {
    // TODO: use unwrap_or_else when the CursorKey trait is changed to allow non-binded CursorValue
    functions::coalesce(A::get_cursor_value(cursor), B::get_cursor_value(cursor))
  }

  fn get_sql_value() -> Self::SqlValue {
    functions::coalesce(A::get_sql_value(), B::get_sql_value())
  }
}

pub struct CoalesceConstKey<const N: i32, A>(pub A);

// TODO: make it generic when `self` parameters are added to this trait
impl<const N: i32, A, C> CursorKey<C> for CoalesceConstKey<N, A>
where
  A: CursorKey<C, SqlType = sql_types::Nullable<sql_types::Integer>>,
{
  type SqlType = sql_types::Integer;
  //type CursorValue = dsl::AsExprOf<i32, sql_types::Integer>;
  type CursorValue =
    functions::coalesce<sql_types::Integer, A::CursorValue, dsl::AsExprOf<i32, sql_types::Integer>>;
  type SqlValue =
    functions::coalesce<sql_types::Integer, A::SqlValue, dsl::AsExprOf<i32, sql_types::Integer>>;

  fn get_cursor_value(cursor: &C) -> Self::CursorValue {
    /* TODO: use this when the CursorKey trait is changed to allow non-binded CursorValue
    A::get_cursor_value(cursor).unwrap_or(N).into_sql::<sql_types::Integer>()*/
    functions::coalesce(
      A::get_cursor_value(cursor),
      N.into_sql::<sql_types::Integer>(),
    )
  }

  fn get_sql_value() -> Self::SqlValue {
    functions::coalesce(A::get_sql_value(), N.into_sql::<sql_types::Integer>())
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
  fn text_if(mut self, text: &str, condition: bool) -> Self {
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

pub fn limit_fetch(limit: Option<i64>) -> LemmyResult<i64> {
  Ok(match limit {
    Some(limit) => {
      if !(1..=FETCH_LIMIT_MAX.try_into()?).contains(&limit) {
        return Err(LemmyErrorType::InvalidFetchLimit.into());
      }
      limit
    }
    None => FETCH_LIMIT_DEFAULT,
  })
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

fn establish_connection(config: &str) -> BoxFuture<'_, ConnectionResult<AsyncPgConnection>> {
  let fut = async {
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
  let db_url = SETTINGS.get_database_url_with_options()?;
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
        Err(HookError::Message("Connection is too old".into()))
      } else {
        Ok(())
      }
    }))
    .build()?;

  lemmy_db_schema_setup::run(lemmy_db_schema_setup::Options::default().run(), &db_url)?;

  Ok(pool)
}

#[allow(clippy::expect_used)]
pub fn build_db_pool_for_tests() -> ActualDbPool {
  build_db_pool().expect("db pool missing")
}

pub mod functions {
  use diesel::sql_types::{Float, Int2, Int4, Nullable, Text};

  define_sql_function! {
    fn get_score(non_1_upvotes: Nullable<Int4>, non_0_downvotes: Nullable<Int4>) -> Int4;
  }

  define_sql_function! {
    fn get_controversy_rank(non_1_upvotes: Nullable<Int4>, non_0_downvotes: Nullable<Int4>) -> Float;
  }

  define_sql_function! {
    fn get_hot_rank(non_1_upvotes: Nullable<Int4>, non_0_downvotes: Nullable<Int4>, age: Nullable<Int2>) -> Float;
  }

  define_sql_function! {
    fn get_scaled_rank(non_1_upvotes: Nullable<Int4>, non_0_downvotes: Nullable<Int4>, age: Nullable<Int2>, non_0_community_interactions_month: Nullable<Int4>) -> Float;
  }

  define_sql_function! {
    fn get_community_hot_rank(non_1_subscribers: Nullable<Int4>, age: Nullable<Int2>) -> Float;
  }

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

/// Output of `IntoSql::into_sql` for a type that implements `AsRecord`
pub type AsRecordOutput<T> = dsl::AsExprOf<T, sql_types::Record<<T as Expression>::SqlType>>;

pub type ResultFuture<'a, T> = BoxFuture<'a, Result<T, DieselError>>;

pub fn paginate<Q, C>(
  query: Q,
  sort_direction: SortDirection,
  page_after: Option<C>,
  page_before_or_equal: Option<C>,
  page_back: Option<bool>,
) -> PaginatedQueryBuilder<C, Q> {
  let mut query = PaginatedQueryBuilder::new(query, sort_direction);

  if page_back.unwrap_or_default() {
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

pub(crate) fn format_actor_url(
  name: &str,
  domain: &str,
  prefix: char,
  settings: &Settings,
) -> LemmyResult<Url> {
  let local_protocol_and_hostname = settings.get_protocol_and_hostname();
  let local_hostname = &settings.hostname;
  let url = if domain != local_hostname {
    format!("{local_protocol_and_hostname}/{prefix}/{name}@{domain}",)
  } else {
    format!("{local_protocol_and_hostname}/{prefix}/{name}")
  };
  Ok(Url::parse(&url)?)
}

/// Make sure the like score is 1, or -1
///
/// Uses a default NotFound error, that you should map to
/// CouldntLikeComment/CouldntLikePost.
pub(crate) fn validate_like(like_score: i16) -> LemmyResult<()> {
  if [-1, 1].contains(&like_score) {
    Ok(())
  } else {
    Err(LemmyErrorType::NotFound.into())
  }
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
