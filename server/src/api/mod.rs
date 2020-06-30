use crate::{
  db::{community::*, community_view::*, moderator::*, site::*, user::*, user_view::*},
  websocket::WebsocketInfo,
  DbPool,
  LemmyError,
};
use actix_web::client::Client;

pub mod comment;
pub mod community;
pub mod post;
pub mod site;
pub mod user;

#[derive(Fail, Debug)]
#[fail(display = "{{\"error\":\"{}\"}}", message)]
pub struct APIError {
  pub message: String,
}

impl APIError {
  pub fn err(msg: &str) -> Self {
    APIError {
      message: msg.to_string(),
    }
  }
}

pub struct Oper<T> {
  data: T,
  client: Client,
}

impl<Data> Oper<Data> {
  pub fn new(data: Data, client: Client) -> Oper<Data> {
    Oper { data, client }
  }
}

#[async_trait::async_trait(?Send)]
pub trait Perform {
  type Response: serde::ser::Serialize + Send;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<Self::Response, LemmyError>;
}
