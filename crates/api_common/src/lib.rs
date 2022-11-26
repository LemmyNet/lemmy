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
use anyhow::Context;
use lemmy_db_schema::{newtypes::DbUrl, source::secret::Secret, utils::DbPool};
use lemmy_utils::{
  error::LemmyError,
  location_info,
  rate_limit::RateLimitCell,
  settings::{structs::Settings, SETTINGS},
};
use reqwest_middleware::ClientWithMiddleware;
use url::{ParseError, Url};

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

pub enum EndpointType {
  Community,
  Person,
  Post,
  Comment,
  PrivateMessage,
}

/// Generates an apub endpoint for a given domain, IE xyz.tld
pub fn generate_local_apub_endpoint(
  endpoint_type: EndpointType,
  name: &str,
  domain: &str,
) -> Result<DbUrl, ParseError> {
  let point = match endpoint_type {
    EndpointType::Community => "c",
    EndpointType::Person => "u",
    EndpointType::Post => "post",
    EndpointType::Comment => "comment",
    EndpointType::PrivateMessage => "private_message",
  };

  Ok(Url::parse(&format!("{}/{}/{}", domain, point, name))?.into())
}

pub fn generate_followers_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{}/followers", actor_id))?.into())
}

pub fn generate_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{}/inbox", actor_id))?.into())
}

pub fn generate_site_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  let mut actor_id: Url = actor_id.clone().into();
  actor_id.set_path("site_inbox");
  Ok(actor_id.into())
}

pub fn generate_shared_inbox_url(actor_id: &DbUrl) -> Result<DbUrl, LemmyError> {
  let actor_id: Url = actor_id.clone().into();
  let url = format!(
    "{}://{}{}/inbox",
    &actor_id.scheme(),
    &actor_id.host_str().context(location_info!())?,
    if let Some(port) = actor_id.port() {
      format!(":{}", port)
    } else {
      String::new()
    },
  );
  Ok(Url::parse(&url)?.into())
}

pub fn generate_outbox_url(actor_id: &DbUrl) -> Result<DbUrl, ParseError> {
  Ok(Url::parse(&format!("{}/outbox", actor_id))?.into())
}

pub fn generate_moderators_url(community_id: &DbUrl) -> Result<DbUrl, LemmyError> {
  Ok(Url::parse(&format!("{}/moderators", community_id))?.into())
}
