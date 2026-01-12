use crate::schema_setup;
use db_pool::{
  PrivilegedPostgresConfig,
  r#async::{
    DatabasePool,
    DatabasePoolBuilderTrait,
    DieselAsyncPostgresBackend,
    DieselDeadpool,
    ReusableConnectionPool,
  },
};
use deadpool::{Runtime, Status};
use diesel::result::{
  ConnectionError,
  ConnectionResult,
  Error::{self as DieselError, QueryBuilderError},
};
use diesel_async::{
  AsyncConnection,
  async_connection_wrapper::AsyncConnectionWrapper,
  pg::AsyncPgConnection,
  pooled_connection::{
    AsyncDieselConnectionManager,
    ManagerConfig,
    deadpool::{Hook, HookError, Object as PooledConnection, Pool},
  },
  scoped_futures::ScopedBoxFuture,
};
use futures_util::{FutureExt, future::BoxFuture};
use lemmy_utils::{
  error::{LemmyError, LemmyResult},
  settings::SETTINGS,
};
use rustls::{
  ClientConfig,
  DigitallySignedStruct,
  SignatureScheme,
  client::danger::{
    DangerousClientConfigBuilder,
    HandshakeSignatureValid,
    ServerCertVerified,
    ServerCertVerifier,
  },
  crypto::{self, verify_tls12_signature, verify_tls13_signature},
  pki_types::{CertificateDer, ServerName, UnixTime},
};
use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
  time::Duration,
};
use tokio::sync::OnceCell;
use tracing::error;
use url::Url;

pub type ActualDbPool = Pool<AsyncPgConnection>;
pub type ReusableDbPool =
  ReusableConnectionPool<'static, DieselAsyncPostgresBackend<DieselDeadpool>>;

/// References a pool or connection. Functions must take `&mut DbPool<'_>` to allow implicit
/// reborrowing.
///
/// https://github.com/rust-lang/rfcs/issues/1403
pub enum DbPool<'a> {
  Pool(&'a ActualDbPool),
  ReusablePool(&'a ReusableDbPool),
  Conn(&'a mut AsyncPgConnection),
}

pub enum DbConn<'a> {
  Pool(PooledConnection<AsyncPgConnection>),
  Conn(&'a mut AsyncPgConnection),
}

#[derive(Clone)]
pub enum GenericDbPool {
  Actual(ActualDbPool),
  Reusable(Arc<ReusableDbPool>),
}

impl GenericDbPool {
  pub fn status(&self) -> Status {
    match self {
      GenericDbPool::Actual(pool) => pool.status(),
      GenericDbPool::Reusable(pool) => pool.status(),
    }
  }
}

pub async fn get_conn<'a, 'b: 'a>(pool: &'a mut DbPool<'b>) -> Result<DbConn<'a>, DieselError> {
  Ok(match pool {
    DbPool::Pool(pool) => DbConn::Pool(pool.get().await.map_err(|e| QueryBuilderError(e.into()))?),
    DbPool::Conn(conn) => DbConn::Conn(conn),
    DbPool::ReusablePool(pool) => {
      DbConn::Pool(pool.get().await.map_err(|e| QueryBuilderError(e.into()))?)
    }
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

impl<'a> From<&'a ReusableDbPool> for DbPool<'a> {
  fn from(value: &'a ReusableDbPool) -> Self {
    DbPool::ReusablePool(value)
  }
}

impl<'a> From<&'a GenericDbPool> for DbPool<'a> {
  fn from(value: &'a GenericDbPool) -> Self {
    match value {
      GenericDbPool::Actual(pool) => DbPool::Pool(pool),
      GenericDbPool::Reusable(pool) => DbPool::ReusablePool(pool),
    }
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
    let _: &mut $crate::connection::DbPool<'_> = $pool;

    match $pool {
      // Run concurrently with `try_join`
      $crate::connection::DbPool::Pool(__pool) => ::futures_util::try_join!(
        $(async {
          let mut __dbpool = $crate::connection::DbPool::Pool(__pool);
          ($func)(&mut __dbpool).await
        }),+
      ),
      // Run sequentially
      $crate::connection::DbPool::Conn(__conn) => async {
        Ok(($({
          let mut __dbpool = $crate::connection::DbPool::Conn(__conn);
          // `?` prevents the error type from being inferred in an `async` block, so `match` is used instead
          match ($func)(&mut __dbpool).await {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__v) => return ::core::result::Result::Err(__v),
          }
        }),+))
      }.await,
      // Run concurrently with `try_join`
      $crate::connection::DbPool::ReusablePool(__pool) => ::futures_util::try_join!(
        $(async {
          let mut __dbpool = $crate::connection::DbPool::ReusablePool(__pool);
          ($func)(&mut __dbpool).await
        }),+
      ),
    }
  }};
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

  crate::schema_setup::run(crate::schema_setup::Options::default().run(), &db_url)?;

  Ok(pool)
}

#[allow(clippy::expect_used)]
pub async fn build_db_pool_for_tests()
-> ReusableConnectionPool<'static, DieselAsyncPostgresBackend<DieselDeadpool>> {
  static POOL: OnceCell<DatabasePool<DieselAsyncPostgresBackend<DieselDeadpool>>> =
    OnceCell::const_new();
  let db_pool = POOL
    .get_or_init(|| async {
      let conn_string = SETTINGS.get_database_url_with_options().unwrap();

      let db_url = Url::parse(conn_string.as_str()).expect("db url");

      let config = PrivilegedPostgresConfig::new()
        .host(db_url.host().expect("db host").to_string())
        .port(db_url.port().expect("db port"))
        .username(db_url.username().to_string())
        .password(Some(db_url.password().expect("db password").to_string()))
        .options(SETTINGS.get_lemmy_connection_options());

      let backend = DieselAsyncPostgresBackend::new(
        config,
        |manager| Pool::builder(manager).max_size(2),
        |manager| Pool::builder(manager).max_size(SETTINGS.database.pool_size),
        None,
        move |conn| {
          Box::pin(async {
            let mut async_wrapper: AsyncConnectionWrapper<AsyncPgConnection> =
              AsyncConnectionWrapper::from(conn);

            tokio::task::spawn_blocking(move || {
              schema_setup::run_with_connection(
                schema_setup::Options::default().run(),
                &mut async_wrapper,
              )
              .expect("run migrations")
            })
            .await
            .expect("task panicked");

            None
          })
        },
      )
      .await
      .expect("diesel postgres backend");

      backend
        .create_database_pool()
        .await
        .expect("create db pool")
    })
    .await;

  db_pool.pull_immutable().await
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
