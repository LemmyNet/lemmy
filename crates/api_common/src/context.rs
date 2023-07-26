use lemmy_db_schema::{
  source::secret::Secret,
  utils::{ActualDbPool, DbPool},
};
use lemmy_utils::{
  email::{DefaultEmailSender, EmailSender},
  rate_limit::RateLimitCell,
  settings::{structs::Settings, SETTINGS},
};
use reqwest_middleware::ClientWithMiddleware;
use std::{ops::Deref, sync::Arc};

#[derive(Clone)]
pub struct LemmyContext {
  pool: ActualDbPool,
  client: Arc<ClientWithMiddleware>,
  secret: Arc<Secret>,
  rate_limit_cell: RateLimitCell,
  email_sender: Arc<dyn EmailSender + Send + Sync>,
}

impl LemmyContext {
  pub fn create(
    pool: ActualDbPool,
    client: ClientWithMiddleware,
    secret: Secret,
    rate_limit_cell: RateLimitCell,
  ) -> LemmyContext {
    let email_sender = Arc::new(DefaultEmailSender {});
    Self::create_non_default(pool, client, secret, rate_limit_cell, email_sender)
  }
  pub fn create_non_default(
    pool: ActualDbPool,
    client: ClientWithMiddleware,
    secret: Secret,
    rate_limit_cell: RateLimitCell,
    email_sender: Arc<dyn EmailSender + Send + Sync>,
  ) -> LemmyContext {
    LemmyContext {
      pool,
      client: Arc::new(client),
      secret: Arc::new(secret),
      rate_limit_cell,
      email_sender,
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
  pub fn settings_updated_channel(&self) -> &RateLimitCell {
    &self.rate_limit_cell
  }
  pub fn email_sender(&self) -> &(dyn EmailSender + Send + Sync) {
    self.email_sender.deref()
  }
}
