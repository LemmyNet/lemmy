use crate::websocket::WebsocketInfo;
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
use failure::Error;

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
}

impl<Data> Oper<Data> {
  pub fn new(data: Data) -> Oper<Data> {
    Oper { data }
  }
}

pub trait Perform {
  type Response: serde::ser::Serialize + Send;

  fn perform(
    &self,
    pool: Pool<ConnectionManager<PgConnection>>,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<Self::Response, Error>;
}
