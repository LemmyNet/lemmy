pub mod comment;
pub mod community;
pub mod person;
pub mod post;
pub mod private_message;
#[cfg(feature = "full")]
pub mod request;
pub mod sensitive;
pub mod site;
#[cfg(feature = "full")]
pub mod utils;
pub mod websocket;

#[macro_use]
extern crate strum_macros;
pub extern crate lemmy_db_schema;
pub extern crate lemmy_db_views;
pub extern crate lemmy_db_views_actor;
pub extern crate lemmy_db_views_moderator;

use crate::websocket::chat_server::ChatServer;
use actix::Addr;
use lemmy_db_schema::{source::secret::Secret, utils::DbPool};
use lemmy_utils::{
  rate_limit::RateLimitCell,
  settings::{structs::Settings, SETTINGS},
};
use reqwest_middleware::ClientWithMiddleware;

pub struct LemmyContext {
  pool: DbPool,
  chat_server: Addr<ChatServer>,
  client: ClientWithMiddleware,
  settings: Settings,
  secret: Secret,
  rate_limit_cell: RateLimitCell,
}

impl LemmyContext {
  pub fn create(
    pool: DbPool,
    chat_server: Addr<ChatServer>,
    client: ClientWithMiddleware,
    settings: Settings,
    secret: Secret,
    settings_updated_channel: RateLimitCell,
  ) -> LemmyContext {
    LemmyContext {
      pool,
      chat_server,
      client,
      settings,
      secret,
      rate_limit_cell: settings_updated_channel,
    }
  }
  pub fn pool(&self) -> &DbPool {
    &self.pool
  }
  pub fn chat_server(&self) -> &Addr<ChatServer> {
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
