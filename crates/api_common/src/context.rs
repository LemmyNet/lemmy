use crate::request::client_builder;
use activitypub_federation::config::{Data, FederationConfig};
use anyhow::anyhow;
use lemmy_db_schema::{
  source::secret::Secret,
  utils::{build_db_pool_for_tests, ActualDbPool, DbPool},
};
use lemmy_utils::{
  rate_limit::RateLimitCell,
  settings::{structs::Settings, SETTINGS},
};
use reqwest::{Request, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Middleware, Next};
use std::sync::Arc;
use task_local_extensions::Extensions;

#[derive(Clone)]
pub struct LemmyContext {
  pool: ActualDbPool,
  client: Arc<ClientWithMiddleware>,
  secret: Arc<Secret>,
  rate_limit_cell: RateLimitCell,
}

impl LemmyContext {
  pub fn create(
    pool: ActualDbPool,
    client: ClientWithMiddleware,
    secret: Secret,
    rate_limit_cell: RateLimitCell,
  ) -> LemmyContext {
    LemmyContext {
      pool,
      client: Arc::new(client),
      secret: Arc::new(secret),
      rate_limit_cell,
    }
  }
  pub fn pool(&self) -> DbPool<'_> {
    DbPool::Pool(&self.pool)
  }
  pub fn inner_pool(&self) -> &ActualDbPool {
    &self.pool
  }
  pub fn client(&self) -> &ClientWithMiddleware {
    &self.client
  }
  pub fn settings(&self) -> &'static Settings {
    &SETTINGS
  }
  pub fn secret(&self) -> &Secret {
    &self.secret
  }
  pub fn rate_limit_cell(&self) -> &RateLimitCell {
    &self.rate_limit_cell
  }

  /// Initialize a context for use in tests, optionally blocks network requests.
  ///
  /// Do not use this in production code.
  pub async fn init_test_context() -> Data<LemmyContext> {
    Self::build_test_context(true).await
  }

  /// Initialize a context for use in tests, with network requests allowed.
  /// TODO: get rid of this if possible.
  ///
  /// Do not use this in production code.
  pub async fn init_test_context_with_networking() -> Data<LemmyContext> {
    Self::build_test_context(false).await
  }

  async fn build_test_context(block_networking: bool) -> Data<LemmyContext> {
    // call this to run migrations
    let pool = build_db_pool_for_tests().await;

    let client = client_builder(&SETTINGS).build().expect("build client");

    let mut client = ClientBuilder::new(client);
    if block_networking {
      client = client.with(BlockedMiddleware);
    }
    let client = client.build();
    let secret = Secret {
      id: 0,
      jwt_secret: String::new(),
    };

    let rate_limit_cell = RateLimitCell::with_test_config();

    let context = LemmyContext::create(pool, client, secret, rate_limit_cell.clone());
    let config = FederationConfig::builder()
      .domain(context.settings().hostname.clone())
      .app_data(context)
      .http_fetch_limit(0)
      .build()
      .await
      .expect("build federation config");
    config.to_request_data()
  }
}

struct BlockedMiddleware;

/// A reqwest middleware which blocks all requests
#[async_trait::async_trait]
impl Middleware for BlockedMiddleware {
  async fn handle(
    &self,
    _req: Request,
    _extensions: &mut Extensions,
    _next: Next<'_>,
  ) -> reqwest_middleware::Result<Response> {
    Err(anyhow!("Network requests not allowed").into())
  }
}
