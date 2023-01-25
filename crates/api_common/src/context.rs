use crate::websocket::chat_server::ChatServer;
use lemmy_db_schema::{source::secret::Secret, utils::DbPool};
use lemmy_utils::{
  rate_limit::RateLimitCell,
  settings::{structs::Settings, SETTINGS},
};
use reqwest_middleware::ClientWithMiddleware;
use std::sync::Arc;

pub struct LemmyContext {
  pool: DbPool,
  chat_server: Arc<ChatServer>,
  client: ClientWithMiddleware,
  settings: Settings,
  secret: Secret,
  rate_limit_cell: RateLimitCell,
}

impl LemmyContext {
  pub fn create(
    pool: DbPool,
    chat_server: Arc<ChatServer>,
    client: ClientWithMiddleware,
    settings: Settings,
    secret: Secret,
    rate_limit_cell: RateLimitCell,
  ) -> LemmyContext {
    LemmyContext {
      pool,
      chat_server,
      client,
      settings,
      secret,
      rate_limit_cell,
    }
  }
  pub fn pool(&self) -> &DbPool {
    &self.pool
  }
  pub fn chat_server(&self) -> &Arc<ChatServer> {
    &self.chat_server
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
}

impl Clone for LemmyContext {
  fn clone(&self) -> Self {
    LemmyContext {
      pool: self.pool.clone(),
      chat_server: self.chat_server.clone(),
      client: self.client.clone(),
      settings: self.settings.clone(),
      secret: self.secret.clone(),
      rate_limit_cell: self.rate_limit_cell.clone(),
    }
  }
}
