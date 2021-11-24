pub mod comment;
pub mod community;
pub mod person;
pub mod post;
pub mod site;
pub mod websocket;

use crate::site::FederatedInstances;
use lemmy_db_schema::{
  newtypes::{CommunityId, LocalUserId, PersonId, PostId},
  source::{
    community::Community,
    person_block::PersonBlock,
    post::{Post, PostRead, PostReadForm},
    secret::Secret,
    site::Site,
  },
  traits::{Crud, Readable},
  DbPool,
};
use lemmy_db_views::local_user_view::{LocalUserSettingsView, LocalUserView};
use lemmy_db_views_actor::{
  community_person_ban_view::CommunityPersonBanView,
  community_view::CommunityView,
};
use lemmy_utils::{claims::Claims, settings::structs::FederationConfig, LemmyError};
use url::Url;

pub async fn blocking<F, T>(pool: &DbPool, f: F) -> Result<T, LemmyError>
where
  F: FnOnce(&diesel::PgConnection) -> T + Send + 'static,
  T: Send + 'static,
{
  let pool = pool.clone();
  let res = actix_web::web::block(move || {
    let conn = pool.get()?;
    let res = (f)(&conn);
    Ok(res) as Result<T, LemmyError>
  })
  .await?;

  res
}

pub async fn is_mod_or_admin(
  pool: &DbPool,
  person_id: PersonId,
  community_id: CommunityId,
) -> Result<(), LemmyError> {
  let is_mod_or_admin = blocking(pool, move |conn| {
    CommunityView::is_mod_or_admin(conn, person_id, community_id)
  })
  .await?;
  if !is_mod_or_admin {
    return Err(LemmyError::from_message("not_a_mod_or_admin".into()));
  }
  Ok(())
}

pub fn is_admin(local_user_view: &LocalUserView) -> Result<(), LemmyError> {
  if !local_user_view.person.admin {
    return Err(LemmyError::from_message("not_an_admin".into()));
  }
  Ok(())
}

pub async fn get_post(post_id: PostId, pool: &DbPool) -> Result<Post, LemmyError> {
  blocking(pool, move |conn| Post::read(conn, post_id))
    .await?
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("couldnt_find_post".into()))
}

pub async fn mark_post_as_read(
  person_id: PersonId,
  post_id: PostId,
  pool: &DbPool,
) -> Result<PostRead, LemmyError> {
  let post_read_form = PostReadForm { post_id, person_id };

  blocking(pool, move |conn| {
    PostRead::mark_as_read(conn, &post_read_form)
  })
  .await?
  .map_err(LemmyError::from)
  .map_err(|e| e.with_message("couldnt_mark_post_as_read".into()))
}

pub async fn mark_post_as_unread(
  person_id: PersonId,
  post_id: PostId,
  pool: &DbPool,
) -> Result<usize, LemmyError> {
  let post_read_form = PostReadForm { post_id, person_id };

  blocking(pool, move |conn| {
    PostRead::mark_as_unread(conn, &post_read_form)
  })
  .await?
  .map_err(LemmyError::from)
  .map_err(|e| e.with_message("couldnt_mark_post_as_read".into()))
}

pub async fn get_local_user_view_from_jwt(
  jwt: &str,
  pool: &DbPool,
  secret: &Secret,
) -> Result<LocalUserView, LemmyError> {
  let claims = Claims::decode(jwt, &secret.jwt_secret)
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("not_logged_in".into()))?
    .claims;
  let local_user_id = LocalUserId(claims.sub);
  let local_user_view =
    blocking(pool, move |conn| LocalUserView::read(conn, local_user_id)).await??;
  // Check for a site ban
  if local_user_view.person.banned {
    return Err(LemmyError::from_message("site_ban".into()));
  }

  // Check for user deletion
  if local_user_view.person.deleted {
    return Err(LemmyError::from_message("deleted".into()));
  }

  check_validator_time(&local_user_view.local_user.validator_time, &claims)?;

  Ok(local_user_view)
}

/// Checks if user's token was issued before user's password reset.
pub fn check_validator_time(
  validator_time: &chrono::NaiveDateTime,
  claims: &Claims,
) -> Result<(), LemmyError> {
  let user_validation_time = validator_time.timestamp();
  if user_validation_time > claims.iat {
    Err(LemmyError::from_message("not_logged_in".into()))
  } else {
    Ok(())
  }
}

pub async fn get_local_user_view_from_jwt_opt(
  jwt: &Option<String>,
  pool: &DbPool,
  secret: &Secret,
) -> Result<Option<LocalUserView>, LemmyError> {
  match jwt {
    Some(jwt) => Ok(Some(get_local_user_view_from_jwt(jwt, pool, secret).await?)),
    None => Ok(None),
  }
}

pub async fn get_local_user_settings_view_from_jwt(
  jwt: &str,
  pool: &DbPool,
  secret: &Secret,
) -> Result<LocalUserSettingsView, LemmyError> {
  let claims = Claims::decode(jwt, &secret.jwt_secret)
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("not_logged_in".into()))?
    .claims;
  let local_user_id = LocalUserId(claims.sub);
  let local_user_view = blocking(pool, move |conn| {
    LocalUserSettingsView::read(conn, local_user_id)
  })
  .await??;
  // Check for a site ban
  if local_user_view.person.banned {
    return Err(LemmyError::from_message("site_ban".into()));
  }

  check_validator_time(&local_user_view.local_user.validator_time, &claims)?;

  Ok(local_user_view)
}

pub async fn get_local_user_settings_view_from_jwt_opt(
  jwt: &Option<String>,
  pool: &DbPool,
  secret: &Secret,
) -> Result<Option<LocalUserSettingsView>, LemmyError> {
  match jwt {
    Some(jwt) => Ok(Some(
      get_local_user_settings_view_from_jwt(jwt, pool, secret).await?,
    )),
    None => Ok(None),
  }
}

pub async fn check_community_ban(
  person_id: PersonId,
  community_id: CommunityId,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  let is_banned =
    move |conn: &'_ _| CommunityPersonBanView::get(conn, person_id, community_id).is_ok();
  if blocking(pool, is_banned).await? {
    Err(LemmyError::from_message("community_ban".into()))
  } else {
    Ok(())
  }
}

pub async fn check_community_deleted_or_removed(
  community_id: CommunityId,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  let community = blocking(pool, move |conn| Community::read(conn, community_id))
    .await?
    .map_err(LemmyError::from)
    .map_err(|e| e.with_message("couldnt_find_community".into()))?;
  if community.deleted || community.removed {
    Err(LemmyError::from_message("deleted".into()))
  } else {
    Ok(())
  }
}

pub fn check_post_deleted_or_removed(post: &Post) -> Result<(), LemmyError> {
  if post.deleted || post.removed {
    Err(LemmyError::from_message("deleted".into()))
  } else {
    Ok(())
  }
}

pub async fn check_person_block(
  my_id: PersonId,
  potential_blocker_id: PersonId,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  let is_blocked = move |conn: &'_ _| PersonBlock::read(conn, potential_blocker_id, my_id).is_ok();
  if blocking(pool, is_blocked).await? {
    Err(LemmyError::from_message("person_block".into()))
  } else {
    Ok(())
  }
}

pub async fn check_downvotes_enabled(score: i16, pool: &DbPool) -> Result<(), LemmyError> {
  if score == -1 {
    let site = blocking(pool, Site::read_simple).await??;
    if !site.enable_downvotes {
      return Err(LemmyError::from_message("downvotes_disabled".into()));
    }
  }
  Ok(())
}

pub async fn build_federated_instances(
  pool: &DbPool,
  federation_config: &FederationConfig,
  hostname: &str,
) -> Result<Option<FederatedInstances>, LemmyError> {
  let federation = federation_config.to_owned();
  if federation.enabled {
    let distinct_communities = blocking(pool, move |conn| {
      Community::distinct_federated_communities(conn)
    })
    .await??;

    let allowed = federation.allowed_instances;
    let blocked = federation.blocked_instances;

    let mut linked = distinct_communities
      .iter()
      .map(|actor_id| Ok(Url::parse(actor_id)?.host_str().unwrap_or("").to_string()))
      .collect::<Result<Vec<String>, LemmyError>>()?;

    if let Some(allowed) = allowed.as_ref() {
      linked.extend_from_slice(allowed);
    }

    if let Some(blocked) = blocked.as_ref() {
      linked.retain(|a| !blocked.contains(a) && !a.eq(hostname));
    }

    // Sort and remove dupes
    linked.sort_unstable();
    linked.dedup();

    Ok(Some(FederatedInstances {
      linked,
      allowed,
      blocked,
    }))
  } else {
    Ok(None)
  }
}

/// Checks the password length
pub fn password_length_check(pass: &str) -> Result<(), LemmyError> {
  if !(10..=60).contains(&pass.len()) {
    Err(LemmyError::from_message("invalid_password".into()))
  } else {
    Ok(())
  }
}

/// Checks the site description length
pub fn site_description_length_check(description: &str) -> Result<(), LemmyError> {
  if description.len() > 150 {
    Err(LemmyError::from_message(
      "site_description_length_overflow".into(),
    ))
  } else {
    Ok(())
  }
}

/// Checks for a honeypot. If this field is filled, fail the rest of the function
pub fn honeypot_check(honeypot: &Option<String>) -> Result<(), LemmyError> {
  if honeypot.is_some() {
    Err(LemmyError::from_message("honeypot_fail".into()))
  } else {
    Ok(())
  }
}
