use crate::{api::claims::Claims, blocking, websocket::WebsocketInfo, DbPool, LemmyError};
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
use lemmy_utils::{slur_check, slurs_vec_to_str};
use thiserror::Error;

pub mod claims;
pub mod comment;
pub mod community;
pub mod post;
pub mod site;
pub mod user;

#[derive(Debug, Error)]
#[error("{{\"error\":\"{message}\"}}")]
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

pub(in crate::api) async fn get_user_from_jwt(
  jwt: &str,
  pool: &DbPool,
) -> Result<User_, LemmyError> {
  let claims = match Claims::decode(&jwt) {
    Ok(claims) => claims.claims,
    Err(_e) => return Err(APIError::err("not_logged_in").into()),
  };
  let user_id = claims.id;
  let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
  // Check for a site ban
  if user.banned {
    return Err(APIError::err("site_ban").into());
  }
  Ok(user)
}

pub(in crate::api) async fn get_user_from_jwt_opt(
  jwt: &Option<String>,
  pool: &DbPool,
) -> Result<Option<User_>, LemmyError> {
  match jwt {
    Some(jwt) => Ok(Some(get_user_from_jwt(jwt, pool).await?)),
    None => Ok(None),
  }
}

pub(in crate::api) fn check_slurs(text: &str) -> Result<(), APIError> {
  if let Err(slurs) = slur_check(text) {
    Err(APIError::err(&slurs_vec_to_str(slurs)))
  } else {
    Ok(())
  }
}
pub(in crate::api) fn check_slurs_opt(text: &Option<String>) -> Result<(), APIError> {
  match text {
    Some(t) => check_slurs(t),
    None => Ok(()),
  }
}
pub(in crate::api) async fn check_community_ban(
  user_id: i32,
  community_id: i32,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  let is_banned = move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
  if blocking(pool, is_banned).await? {
    Err(APIError::err("community_ban").into())
  } else {
    Ok(())
  }
}
