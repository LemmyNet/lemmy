use crate::claims::Claims;
use actix_web::{web, web::Data};
use anyhow::anyhow;
use lemmy_db::{
  community::Community,
  community_view::CommunityUserBanView,
  post::Post,
  user::User_,
  Crud,
  DbPool,
};
use lemmy_structs::{blocking, comment::*, community::*, post::*, site::*, user::*};
use lemmy_utils::{
  apub::get_apub_protocol_string,
  request::{retry, RecvError},
  settings::Settings,
  APIError,
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::{serialize_websocket_message, LemmyContext, UserOperation};
use log::error;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::Client;
use serde::Deserialize;
use std::process::Command;

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

pub(in crate) async fn is_mod_or_admin(
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

pub(in crate) async fn get_post(post_id: i32, pool: &DbPool) -> Result<Post, LemmyError> {
  match blocking(pool, move |conn| Post::read(conn, post_id)).await? {
    Ok(post) => Ok(post),
    Err(_e) => Err(APIError::err("couldnt_find_post").into()),
  }
}

pub(in crate) async fn get_user_from_jwt(jwt: &str, pool: &DbPool) -> Result<User_, LemmyError> {
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

pub(in crate) async fn get_user_from_jwt_opt(
  jwt: &Option<String>,
  pool: &DbPool,
) -> Result<Option<User_>, LemmyError> {
  match jwt {
    Some(jwt) => Ok(Some(get_user_from_jwt(jwt, pool).await?)),
    None => Ok(None),
  }
}

pub(in crate) async fn check_community_ban(
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
    UserOperation::SaveUserSettings => {
      do_websocket_operation::<SaveUserSettings>(context, id, op, data).await
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

#[derive(Deserialize, Debug)]
pub(crate) struct IframelyResponse {
  title: Option<String>,
  description: Option<String>,
  thumbnail_url: Option<String>,
  html: Option<String>,
}

pub(crate) async fn fetch_iframely(
  client: &Client,
  url: &str,
) -> Result<IframelyResponse, LemmyError> {
  let fetch_url = format!("http://iframely/oembed?url={}", url);

  let response = retry(|| client.get(&fetch_url).send()).await?;

  let res: IframelyResponse = response
    .json()
    .await
    .map_err(|e| RecvError(e.to_string()))?;
  Ok(res)
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PictrsResponse {
  files: Vec<PictrsFile>,
  msg: String,
}

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct PictrsFile {
  file: String,
  delete_token: String,
}

pub(crate) async fn fetch_pictrs(
  client: &Client,
  image_url: &str,
) -> Result<PictrsResponse, LemmyError> {
  is_image_content_type(client, image_url).await?;

  let fetch_url = format!(
    "http://pictrs:8080/image/download?url={}",
    utf8_percent_encode(image_url, NON_ALPHANUMERIC) // TODO this might not be needed
  );

  let response = retry(|| client.get(&fetch_url).send()).await?;

  let response: PictrsResponse = response
    .json()
    .await
    .map_err(|e| RecvError(e.to_string()))?;

  if response.msg == "ok" {
    Ok(response)
  } else {
    Err(anyhow!("{}", &response.msg).into())
  }
}

async fn fetch_iframely_and_pictrs_data(
  client: &Client,
  url: Option<String>,
) -> (
  Option<String>,
  Option<String>,
  Option<String>,
  Option<String>,
) {
  match &url {
    Some(url) => {
      // Fetch iframely data
      let (iframely_title, iframely_description, iframely_thumbnail_url, iframely_html) =
        match fetch_iframely(client, url).await {
          Ok(res) => (res.title, res.description, res.thumbnail_url, res.html),
          Err(e) => {
            error!("iframely err: {}", e);
            (None, None, None, None)
          }
        };

      // Fetch pictrs thumbnail
      let pictrs_hash = match iframely_thumbnail_url {
        Some(iframely_thumbnail_url) => match fetch_pictrs(client, &iframely_thumbnail_url).await {
          Ok(res) => Some(res.files[0].file.to_owned()),
          Err(e) => {
            error!("pictrs err: {}", e);
            None
          }
        },
        // Try to generate a small thumbnail if iframely is not supported
        None => match fetch_pictrs(client, &url).await {
          Ok(res) => Some(res.files[0].file.to_owned()),
          Err(e) => {
            error!("pictrs err: {}", e);
            None
          }
        },
      };

      // The full urls are necessary for federation
      let pictrs_thumbnail = if let Some(pictrs_hash) = pictrs_hash {
        Some(format!(
          "{}://{}/pictrs/image/{}",
          get_apub_protocol_string(),
          Settings::get().hostname,
          pictrs_hash
        ))
      } else {
        None
      };

      (
        iframely_title,
        iframely_description,
        iframely_html,
        pictrs_thumbnail,
      )
    }
    None => (None, None, None, None),
  }
}

pub(crate) async fn is_image_content_type(client: &Client, test: &str) -> Result<(), LemmyError> {
  let response = retry(|| client.get(test).send()).await?;

  if response
    .headers()
    .get("Content-Type")
    .ok_or_else(|| anyhow!("No Content-Type header"))?
    .to_str()?
    .starts_with("image/")
  {
    Ok(())
  } else {
    Err(anyhow!("Not an image type.").into())
  }
}

#[cfg(test)]
mod tests {
  use crate::{captcha_espeak_wav_base64, is_image_content_type};

  #[test]
  fn test_image() {
    actix_rt::System::new("tset_image").block_on(async move {
      let client = reqwest::Client::default();
      assert!(is_image_content_type(&client, "https://1734811051.rsc.cdn77.org/data/images/full/365645/as-virus-kills-navajos-in-their-homes-tribal-women-provide-lifeline.jpg?w=600?w=650").await.is_ok());
      assert!(is_image_content_type(&client,
                                    "https://twitter.com/BenjaminNorton/status/1259922424272957440?s=20"
      )
        .await.is_err()
      );
    });
  }

  #[test]
  fn test_espeak() {
    assert!(captcha_espeak_wav_base64("WxRt2l").is_ok())
  }

  // These helped with testing
  // #[test]
  // fn test_iframely() {
  //   let res = fetch_iframely(client, "https://www.redspark.nu/?p=15341").await;
  //   assert!(res.is_ok());
  // }

  // #[test]
  // fn test_pictshare() {
  //   let res = fetch_pictshare("https://upload.wikimedia.org/wikipedia/en/2/27/The_Mandalorian_logo.jpg");
  //   assert!(res.is_ok());
  //   let res_other = fetch_pictshare("https://upload.wikimedia.org/wikipedia/en/2/27/The_Mandalorian_logo.jpgaoeu");
  //   assert!(res_other.is_err());
  // }
}
