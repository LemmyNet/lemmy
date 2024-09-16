use crate::request::client_builder;
use activitypub_federation::config::{Data, FederationConfig};
use lemmy_db_schema::{
  source::secret::Secret,
  utils::{build_db_pool_for_tests, ActualDbPool, DbPool},
};
use lemmy_utils::{
  rate_limit::RateLimitCell,
  settings::{structs::Settings, SETTINGS},
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use std::sync::Arc;

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

  /// Initialize a context for use in tests which blocks federation network calls.
  ///
  /// Do not use this in production code.
  pub async fn init_test_federation_config() -> FederationConfig<LemmyContext> {
    // call this to run migrations
    let pool = build_db_pool_for_tests().await;

    let client = client_builder(&SETTINGS).build().expect("build client");

    let client = ClientBuilder::new(client).build();
    let secret = Secret {
      id: 0,
      jwt_secret: String::new().into(),
    };

    let rate_limit_cell = RateLimitCell::with_test_config();

    let context = LemmyContext::create(pool, client, secret, rate_limit_cell.clone());

    FederationConfig::builder()
      .domain(context.settings().hostname.clone())
      .app_data(context)
      .debug(true)
      // Dont allow any network fetches
      .http_fetch_limit(0)
      .build()
      .await
      .expect("build federation config")
  }
  pub async fn init_test_context() -> Data<LemmyContext> {
    let config = Self::init_test_federation_config().await;
    config.to_request_data()
  }
}
