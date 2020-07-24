use crate::{blocking, websocket::WebsocketInfo, DbPool, LemmyError};
use actix_web::client::Client;
use lemmy_db::{
  community::*,
  community_view::*,
  moderator::*,
  site::*,
  user::*,
  user_view::*,
  Crud,
};

pub mod claims;
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

pub async fn is_mod_or_admin(
  pool: &DbPool,
  user_id: i32,
  community_id: i32,
) -> Result<(), LemmyError> {
  let is_mod_or_admin = blocking(pool, move |conn| {
    Community::is_mod_or_admin(conn, user_id, community_id)
  })
  .await?;
  if !is_mod_or_admin {
    return Err(APIError::err("not_an_admin").into());
  }
  Ok(())
}
pub async fn is_admin(pool: &DbPool, user_id: i32) -> Result<(), LemmyError> {
  let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
  if !user.admin {
    return Err(APIError::err("not_an_admin").into());
  }
  Ok(())
}
