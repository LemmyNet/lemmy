use crate::request::client_builder;
use activitypub_federation::config::{Data, FederationConfig};
use lemmy_db_schema::source::secret::Secret;
use lemmy_diesel_utils::connection::{DbPool, GenericDbPool, build_db_pool_for_tests};
use lemmy_utils::{
  rate_limit::RateLimit,
  settings::{SETTINGS, structs::Settings},
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use std::sync::Arc;

#[derive(Clone)]
pub struct LemmyContext {
  pool: GenericDbPool,
  client: Arc<ClientWithMiddleware>,
  /// Pictrs requests must bypass proxy. Unfortunately no_proxy can only be set on ClientBuilder
  /// and not on RequestBuilder, so we need a separate client here.
  pictrs_client: Arc<ClientWithMiddleware>,
  secret: Arc<Secret>,
  rate_limit_cell: RateLimit,
}

impl LemmyContext {
  pub fn create(
    pool: GenericDbPool,
    client: ClientWithMiddleware,
    pictrs_client: ClientWithMiddleware,
    secret: Secret,
    rate_limit_cell: RateLimit,
  ) -> LemmyContext {
    LemmyContext {
      pool,
      client: Arc::new(client),
      pictrs_client: Arc::new(pictrs_client),
      secret: Arc::new(secret),
      rate_limit_cell,
    }
  }
  pub fn pool(&self) -> DbPool<'_> {
    match &self.pool {
      GenericDbPool::Actual(pool) => DbPool::Pool(pool),
      GenericDbPool::Reusable(pool) => DbPool::ReusablePool(pool),
    }
  }
  pub fn inner_pool(&self) -> &GenericDbPool {
    &self.pool
  }
  pub fn client(&self) -> &ClientWithMiddleware {
    &self.client
  }
  pub fn pictrs_client(&self) -> &ClientWithMiddleware {
    &self.pictrs_client
  }
  pub fn settings(&self) -> &'static Settings {
    &SETTINGS
  }
  pub fn secret(&self) -> &Secret {
    &self.secret
  }
  pub fn rate_limit_cell(&self) -> &RateLimit {
    &self.rate_limit_cell
  }

  /// Initialize a context for use in tests which blocks federation network calls.
  ///
  /// Do not use this in production code.
  #[allow(clippy::expect_used)]
  pub async fn init_test_federation_config() -> FederationConfig<LemmyContext> {
    // call this to run migrations
    let pool = build_db_pool_for_tests().await;

    let client = client_builder(&SETTINGS).build().expect("build client");

    let client = ClientBuilder::new(client).build();
    let secret = Secret {
      id: 0,
      jwt_secret: String::new().into(),
    };

    let rate_limit_cell = RateLimit::with_debug_config();

    let context = LemmyContext::create(
      GenericDbPool::Reusable(Arc::new(pool)),
      client.clone(),
      client,
      secret,
      rate_limit_cell.clone(),
    );

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
