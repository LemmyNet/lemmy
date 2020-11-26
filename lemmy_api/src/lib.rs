use crate::claims::Claims;
use actix_web::{web, web::Data};
use lemmy_db::{
  community::{Community, CommunityModerator},
  community_view::CommunityUserBanView,
  post::Post,
  user::User_,
  Crud,
  DbPool,
};
use lemmy_structs::{blocking, comment::*, community::*, post::*, site::*, user::*};
use lemmy_utils::{settings::Settings, APIError, ConnectionId, LemmyError};
use lemmy_websocket::{serialize_websocket_message, LemmyContext, UserOperation};
use serde::Deserialize;
use std::process::Command;
use url::Url;

pub mod claims;
pub mod comment;
pub mod community;
pub mod post;
pub mod site;
pub mod user;
pub mod version;

#[async_trait::async_trait(?Send)]
pub trait Perform {
  type Response: serde::ser::Serialize + Send;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError>;
}

pub(crate) async fn is_mod_or_admin(
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

pub(crate) async fn get_post(post_id: i32, pool: &DbPool) -> Result<Post, LemmyError> {
  match blocking(pool, move |conn| Post::read(conn, post_id)).await? {
    Ok(post) => Ok(post),
    Err(_e) => Err(APIError::err("couldnt_find_post").into()),
  }
}

pub(crate) async fn get_user_from_jwt(jwt: &str, pool: &DbPool) -> Result<User_, LemmyError> {
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

pub(crate) async fn get_user_from_jwt_opt(
  jwt: &Option<String>,
  pool: &DbPool,
) -> Result<Option<User_>, LemmyError> {
  match jwt {
    Some(jwt) => Ok(Some(get_user_from_jwt(jwt, pool).await?)),
    None => Ok(None),
  }
}

pub(crate) async fn check_community_ban(
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

/// Returns a list of communities that the user moderates
/// or if a community_id is supplied validates the user is a moderator
/// of that community and returns the community id in a vec
///
/// * `user_id` - the user id of the moderator
/// * `community_id` - optional community id to check for moderator privileges
/// * `pool` - the diesel db pool
pub(crate) async fn collect_moderated_communities(
  user_id: i32,
  community_id: Option<i32>,
  pool: &DbPool,
) -> Result<Vec<i32>, LemmyError> {
  if let Some(community_id) = community_id {
    // if the user provides a community_id, just check for mod/admin privileges
    is_mod_or_admin(pool, user_id, community_id).await?;
    Ok(vec![community_id])
  } else {
    let ids = blocking(pool, move |conn: &'_ _| {
      CommunityModerator::get_user_moderated_communities(conn, user_id)
    })
    .await??;
    Ok(ids)
  }
}

pub(crate) fn check_optional_url(item: &Option<Option<String>>) -> Result<(), LemmyError> {
  if let Some(Some(item)) = &item {
    if Url::parse(item).is_err() {
      return Err(APIError::err("invalid_url").into());
    }
  }
  Ok(())
}

pub(crate) async fn linked_instances(pool: &DbPool) -> Result<Vec<String>, LemmyError> {
  let mut instances: Vec<String> = Vec::new();

  if Settings::get().federation.enabled {
    let distinct_communities = blocking(pool, move |conn| {
      Community::distinct_federated_communities(conn)
    })
    .await??;

    instances = distinct_communities
      .iter()
      .map(|actor_id| Ok(Url::parse(actor_id)?.host_str().unwrap_or("").to_string()))
      .collect::<Result<Vec<String>, LemmyError>>()?;

    instances.append(&mut Settings::get().get_allowed_instances());
    instances.retain(|a| {
      !Settings::get().get_blocked_instances().contains(a)
        && !a.eq("")
        && !a.eq(&Settings::get().hostname)
    });

    // Sort and remove dupes
    instances.sort_unstable();
    instances.dedup();
  }

  Ok(instances)
}

pub async fn match_websocket_operation(
  context: LemmyContext,
  id: ConnectionId,
  op: UserOperation,
  data: &str,
) -> Result<String, LemmyError> {
  match op {
    // User ops
    UserOperation::Login => do_websocket_operation::<Login>(context, id, op, data).await,
    UserOperation::Register => do_websocket_operation::<Register>(context, id, op, data).await,
    UserOperation::GetCaptcha => do_websocket_operation::<GetCaptcha>(context, id, op, data).await,
    UserOperation::GetUserDetails => {
      do_websocket_operation::<GetUserDetails>(context, id, op, data).await
    }
    UserOperation::GetReplies => do_websocket_operation::<GetReplies>(context, id, op, data).await,
    UserOperation::AddAdmin => do_websocket_operation::<AddAdmin>(context, id, op, data).await,
    UserOperation::BanUser => do_websocket_operation::<BanUser>(context, id, op, data).await,
    UserOperation::GetUserMentions => {
      do_websocket_operation::<GetUserMentions>(context, id, op, data).await
    }
    UserOperation::MarkUserMentionAsRead => {
      do_websocket_operation::<MarkUserMentionAsRead>(context, id, op, data).await
    }
    UserOperation::MarkAllAsRead => {
      do_websocket_operation::<MarkAllAsRead>(context, id, op, data).await
    }
    UserOperation::DeleteAccount => {
      do_websocket_operation::<DeleteAccount>(context, id, op, data).await
    }
    UserOperation::PasswordReset => {
      do_websocket_operation::<PasswordReset>(context, id, op, data).await
    }
    UserOperation::PasswordChange => {
      do_websocket_operation::<PasswordChange>(context, id, op, data).await
    }
    UserOperation::UserJoin => do_websocket_operation::<UserJoin>(context, id, op, data).await,
    UserOperation::PostJoin => do_websocket_operation::<PostJoin>(context, id, op, data).await,
    UserOperation::CommunityJoin => {
      do_websocket_operation::<CommunityJoin>(context, id, op, data).await
    }
    UserOperation::ModJoin => do_websocket_operation::<ModJoin>(context, id, op, data).await,
    UserOperation::SaveUserSettings => {
      do_websocket_operation::<SaveUserSettings>(context, id, op, data).await
    }
    UserOperation::GetReportCount => {
      do_websocket_operation::<GetReportCount>(context, id, op, data).await
    }

    // Private Message ops
    UserOperation::CreatePrivateMessage => {
      do_websocket_operation::<CreatePrivateMessage>(context, id, op, data).await
    }
    UserOperation::EditPrivateMessage => {
      do_websocket_operation::<EditPrivateMessage>(context, id, op, data).await
    }
    UserOperation::DeletePrivateMessage => {
      do_websocket_operation::<DeletePrivateMessage>(context, id, op, data).await
    }
    UserOperation::MarkPrivateMessageAsRead => {
      do_websocket_operation::<MarkPrivateMessageAsRead>(context, id, op, data).await
    }
    UserOperation::GetPrivateMessages => {
      do_websocket_operation::<GetPrivateMessages>(context, id, op, data).await
    }

    // Site ops
    UserOperation::GetModlog => do_websocket_operation::<GetModlog>(context, id, op, data).await,
    UserOperation::CreateSite => do_websocket_operation::<CreateSite>(context, id, op, data).await,
    UserOperation::EditSite => do_websocket_operation::<EditSite>(context, id, op, data).await,
    UserOperation::GetSite => do_websocket_operation::<GetSite>(context, id, op, data).await,
    UserOperation::GetSiteConfig => {
      do_websocket_operation::<GetSiteConfig>(context, id, op, data).await
    }
    UserOperation::SaveSiteConfig => {
      do_websocket_operation::<SaveSiteConfig>(context, id, op, data).await
    }
    UserOperation::Search => do_websocket_operation::<Search>(context, id, op, data).await,
    UserOperation::TransferCommunity => {
      do_websocket_operation::<TransferCommunity>(context, id, op, data).await
    }
    UserOperation::TransferSite => {
      do_websocket_operation::<TransferSite>(context, id, op, data).await
    }
    UserOperation::ListCategories => {
      do_websocket_operation::<ListCategories>(context, id, op, data).await
    }

    // Community ops
    UserOperation::GetCommunity => {
      do_websocket_operation::<GetCommunity>(context, id, op, data).await
    }
    UserOperation::ListCommunities => {
      do_websocket_operation::<ListCommunities>(context, id, op, data).await
    }
    UserOperation::CreateCommunity => {
      do_websocket_operation::<CreateCommunity>(context, id, op, data).await
    }
    UserOperation::EditCommunity => {
      do_websocket_operation::<EditCommunity>(context, id, op, data).await
    }
    UserOperation::DeleteCommunity => {
      do_websocket_operation::<DeleteCommunity>(context, id, op, data).await
    }
    UserOperation::RemoveCommunity => {
      do_websocket_operation::<RemoveCommunity>(context, id, op, data).await
    }
    UserOperation::FollowCommunity => {
      do_websocket_operation::<FollowCommunity>(context, id, op, data).await
    }
    UserOperation::GetFollowedCommunities => {
      do_websocket_operation::<GetFollowedCommunities>(context, id, op, data).await
    }
    UserOperation::BanFromCommunity => {
      do_websocket_operation::<BanFromCommunity>(context, id, op, data).await
    }
    UserOperation::AddModToCommunity => {
      do_websocket_operation::<AddModToCommunity>(context, id, op, data).await
    }

    // Post ops
    UserOperation::CreatePost => do_websocket_operation::<CreatePost>(context, id, op, data).await,
    UserOperation::GetPost => do_websocket_operation::<GetPost>(context, id, op, data).await,
    UserOperation::GetPosts => do_websocket_operation::<GetPosts>(context, id, op, data).await,
    UserOperation::EditPost => do_websocket_operation::<EditPost>(context, id, op, data).await,
    UserOperation::DeletePost => do_websocket_operation::<DeletePost>(context, id, op, data).await,
    UserOperation::RemovePost => do_websocket_operation::<RemovePost>(context, id, op, data).await,
    UserOperation::LockPost => do_websocket_operation::<LockPost>(context, id, op, data).await,
    UserOperation::StickyPost => do_websocket_operation::<StickyPost>(context, id, op, data).await,
    UserOperation::CreatePostLike => {
      do_websocket_operation::<CreatePostLike>(context, id, op, data).await
    }
    UserOperation::SavePost => do_websocket_operation::<SavePost>(context, id, op, data).await,
    UserOperation::CreatePostReport => {
      do_websocket_operation::<CreatePostReport>(context, id, op, data).await
    }
    UserOperation::ListPostReports => {
      do_websocket_operation::<ListPostReports>(context, id, op, data).await
    }
    UserOperation::ResolvePostReport => {
      do_websocket_operation::<ResolvePostReport>(context, id, op, data).await
    }

    // Comment ops
    UserOperation::CreateComment => {
      do_websocket_operation::<CreateComment>(context, id, op, data).await
    }
    UserOperation::EditComment => {
      do_websocket_operation::<EditComment>(context, id, op, data).await
    }
    UserOperation::DeleteComment => {
      do_websocket_operation::<DeleteComment>(context, id, op, data).await
    }
    UserOperation::RemoveComment => {
      do_websocket_operation::<RemoveComment>(context, id, op, data).await
    }
    UserOperation::MarkCommentAsRead => {
      do_websocket_operation::<MarkCommentAsRead>(context, id, op, data).await
    }
    UserOperation::SaveComment => {
      do_websocket_operation::<SaveComment>(context, id, op, data).await
    }
    UserOperation::GetComments => {
      do_websocket_operation::<GetComments>(context, id, op, data).await
    }
    UserOperation::CreateCommentLike => {
      do_websocket_operation::<CreateCommentLike>(context, id, op, data).await
    }
    UserOperation::CreateCommentReport => {
      do_websocket_operation::<CreateCommentReport>(context, id, op, data).await
    }
    UserOperation::ListCommentReports => {
      do_websocket_operation::<ListCommentReports>(context, id, op, data).await
    }
    UserOperation::ResolveCommentReport => {
      do_websocket_operation::<ResolveCommentReport>(context, id, op, data).await
    }
  }
}

async fn do_websocket_operation<'a, 'b, Data>(
  context: LemmyContext,
  id: ConnectionId,
  op: UserOperation,
  data: &str,
) -> Result<String, LemmyError>
where
  for<'de> Data: Deserialize<'de> + 'a,
  Data: Perform,
{
  let parsed_data: Data = serde_json::from_str(&data)?;
  let res = parsed_data
    .perform(&web::Data::new(context), Some(id))
    .await?;
  serialize_websocket_message(&op, &res)
}

pub(crate) fn captcha_espeak_wav_base64(captcha: &str) -> Result<String, LemmyError> {
  let mut built_text = String::new();

  // Building proper speech text for espeak
  for mut c in captcha.chars() {
    let new_str = if c.is_alphabetic() {
      if c.is_lowercase() {
        c.make_ascii_uppercase();
        format!("lower case {} ... ", c)
      } else {
        c.make_ascii_uppercase();
        format!("capital {} ... ", c)
      }
    } else {
      format!("{} ...", c)
    };

    built_text.push_str(&new_str);
  }

  espeak_wav_base64(&built_text)
}

pub(crate) fn espeak_wav_base64(text: &str) -> Result<String, LemmyError> {
  // Make a temp file path
  let uuid = uuid::Uuid::new_v4().to_string();
  let file_path = format!("/tmp/lemmy_espeak_{}.wav", &uuid);

  // Write the wav file
  Command::new("espeak")
    .arg("-w")
    .arg(&file_path)
    .arg(text)
    .status()?;

  // Read the wav file bytes
  let bytes = std::fs::read(&file_path)?;

  // Delete the file
  std::fs::remove_file(file_path)?;

  // Convert to base64
  let base64 = base64::encode(bytes);

  Ok(base64)
}

#[cfg(test)]
mod tests {
  use crate::captcha_espeak_wav_base64;

  #[test]
  fn test_espeak() {
    assert!(captcha_espeak_wav_base64("WxRt2l").is_ok())
  }
}
