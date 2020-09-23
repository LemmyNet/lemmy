use crate::{api::claims::Claims, DbPool, LemmyContext};
use actix_web::web::Data;
use lemmy_db::{
  community::Community,
  community_view::CommunityUserBanView,
  post::Post,
  user::User_,
  Crud,
};
use lemmy_structs::blocking;
use lemmy_utils::{settings::Settings, APIError, ConnectionId, LemmyError};
use url::Url;

pub mod claims;
pub mod comment;
pub mod community;
pub mod post;
pub mod site;
pub mod user;

#[async_trait::async_trait(?Send)]
pub trait Perform {
  type Response: serde::ser::Serialize + Send;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError>;
}

pub(in crate::api) async fn is_mod_or_admin(
  pool: &DbPool,
  user_id: i32,
  community_id: i32,
) -> Result<(), LemmyError> {
  let is_mod_or_admin = blocking(pool, move |conn| {
    Community::is_mod_or_admin(conn, user_id, community_id)
  })
  .await?;
  if !is_mod_or_admin {
    return Err(APIError::err("not_a_mod_or_admin").into());
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

pub(in crate::api) async fn get_post(post_id: i32, pool: &DbPool) -> Result<Post, LemmyError> {
  match blocking(pool, move |conn| Post::read(conn, post_id)).await? {
    Ok(post) => Ok(post),
    Err(_e) => Err(APIError::err("couldnt_find_post").into()),
  }
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

pub(in crate::api) async fn linked_instances(pool: &DbPool) -> Result<Vec<String>, LemmyError> {
  let distinct_communities = blocking(pool, move |conn| {
    Community::distinct_federated_communities(conn)
  })
  .await??;

  let mut instances: Vec<String> = Vec::new();

  for actor_id in &distinct_communities {
    instances.push(Url::parse(actor_id)?.host_str().unwrap_or("").to_string());
  }

  let mut allowed_instances = Settings::get().get_allowed_instances();
  instances.append(&mut allowed_instances);

  let blocked_instances = Settings::get().get_blocked_instances();
  for blocked in &blocked_instances {
    instances.retain(|i| !i.eq(blocked));
  }

  // Sort and remove dupes
  instances.sort_unstable();
  instances.dedup();

  // Remove the empties
  instances.retain(|d| !d.eq(""));

  Ok(instances)
}
