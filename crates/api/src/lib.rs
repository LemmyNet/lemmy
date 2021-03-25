use actix_web::{web, web::Data};
use lemmy_api_common::{comment::*, community::*, person::*, post::*, site::*, websocket::*};
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::{serialize_websocket_message, LemmyContext, UserOperation};
use serde::Deserialize;
use std::{env, process::Command};

mod comment;
mod comment_report;
mod community;
mod local_user;
mod post;
mod post_report;
mod private_message;
pub mod routes;
mod site;
mod websocket;

#[async_trait::async_trait(?Send)]
pub trait Perform {
  type Response: serde::ser::Serialize + Send;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError>;
}

pub async fn match_websocket_operation(
  context: LemmyContext,
  id: ConnectionId,
  op: UserOperation,
  data: &str,
) -> Result<String, LemmyError> {
  //TODO: handle commented out actions in crud crate

  match op {
    // User ops
    UserOperation::Login => {
      //do_websocket_operation::<Login>(context, id, op, data).await
      todo!()
    }
    UserOperation::Register => {
      //do_websocket_operation::<Register>(context, id, op, data).await
      todo!()
    }
    UserOperation::GetCaptcha => do_websocket_operation::<GetCaptcha>(context, id, op, data).await,
    UserOperation::GetPersonDetails => {
      //do_websocket_operation::<GetPersonDetails>(context, id, op, data).await
      todo!()
    }
    UserOperation::GetReplies => do_websocket_operation::<GetReplies>(context, id, op, data).await,
    UserOperation::AddAdmin => do_websocket_operation::<AddAdmin>(context, id, op, data).await,
    UserOperation::BanPerson => do_websocket_operation::<BanPerson>(context, id, op, data).await,
    UserOperation::GetPersonMentions => {
      //do_websocket_operation::<GetPersonMentions>(context, id, op, data).await
      todo!()
    }
    UserOperation::MarkPersonMentionAsRead => {
      do_websocket_operation::<MarkPersonMentionAsRead>(context, id, op, data).await
    }
    UserOperation::MarkAllAsRead => {
      do_websocket_operation::<MarkAllAsRead>(context, id, op, data).await
    }
    UserOperation::DeleteAccount => {
      //do_websocket_operation::<DeleteAccount>(context, id, op, data).await
      todo!()
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
      //do_websocket_operation::<CreatePrivateMessage>(context, id, op, data).await
      todo!()
    }
    UserOperation::EditPrivateMessage => {
      //do_websocket_operation::<EditPrivateMessage>(context, id, op, data).await
      todo!()
    }
    UserOperation::DeletePrivateMessage => {
      //do_websocket_operation::<DeletePrivateMessage>(context, id, op, data).await
      todo!()
    }
    UserOperation::MarkPrivateMessageAsRead => {
      do_websocket_operation::<MarkPrivateMessageAsRead>(context, id, op, data).await
    }
    UserOperation::GetPrivateMessages => {
      //do_websocket_operation::<GetPrivateMessages>(context, id, op, data).await
      todo!()
    }

    // Site ops
    UserOperation::GetModlog => do_websocket_operation::<GetModlog>(context, id, op, data).await,
    UserOperation::CreateSite => {
      //do_websocket_operation::<CreateSite>(context, id, op, data).await
      todo!()
    }
    UserOperation::EditSite => {
      //do_websocket_operation::<EditSite>(context, id, op, data).await
      todo!()
    }
    UserOperation::GetSite => {
      //do_websocket_operation::<GetSite>(context, id, op, data).await
      todo!()
    }
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

    // Community ops
    UserOperation::GetCommunity => {
      //do_websocket_operation::<GetCommunity>(context, id, op, data).await
      todo!()
    }
    UserOperation::ListCommunities => {
      //do_websocket_operation::<ListCommunities>(context, id, op, data).await
      todo!()
    }
    UserOperation::CreateCommunity => {
      //do_websocket_operation::<CreateCommunity>(context, id, op, data).await
      todo!()
    }
    UserOperation::EditCommunity => {
      //do_websocket_operation::<EditCommunity>(context, id, op, data).await
      todo!()
    }
    UserOperation::DeleteCommunity => {
      //do_websocket_operation::<DeleteCommunity>(context, id, op, data).await
      todo!()
    }
    UserOperation::RemoveCommunity => {
      //do_websocket_operation::<RemoveCommunity>(context, id, op, data).await
      todo!()
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
    UserOperation::CreatePost => {
      //do_websocket_operation::<CreatePost>(context, id, op, data).await
      todo!()
    }
    UserOperation::GetPost => {
      //do_websocket_operation::<GetPost>(context, id, op, data).await
      todo!()
    }
    UserOperation::GetPosts => {
      //do_websocket_operation::<GetPosts>(context, id, op, data).await
      todo!()
    }
    UserOperation::EditPost => {
      //do_websocket_operation::<EditPost>(context, id, op, data).await
      todo!()
    }
    UserOperation::DeletePost => {
      //do_websocket_operation::<DeletePost>(context, id, op, data).await
      todo!()
    }
    UserOperation::RemovePost => {
      //do_websocket_operation::<RemovePost>(context, id, op, data).await
      todo!()
    }
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
      //do_websocket_operation::<CreateComment>(context, id, op, data).await
      todo!()
    }
    UserOperation::EditComment => {
      //do_websocket_operation::<EditComment>(context, id, op, data).await
      todo!()
    }
    UserOperation::DeleteComment => {
      //do_websocket_operation::<DeleteComment>(context, id, op, data).await
      todo!()
    }
    UserOperation::RemoveComment => {
      //do_websocket_operation::<RemoveComment>(context, id, op, data).await
      todo!()
    }
    UserOperation::MarkCommentAsRead => {
      do_websocket_operation::<MarkCommentAsRead>(context, id, op, data).await
    }
    UserOperation::SaveComment => {
      do_websocket_operation::<SaveComment>(context, id, op, data).await
    }
    UserOperation::GetComments => {
      //do_websocket_operation::<GetComments>(context, id, op, data).await
      todo!()
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
  let file_path = format!(
    "{}/lemmy_espeak_{}.wav",
    env::temp_dir().to_string_lossy(),
    &uuid
  );

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
  use crate::{captcha_espeak_wav_base64, check_validator_time};
  use lemmy_api_common::check_validator_time;
  use lemmy_db_queries::{establish_unpooled_connection, source::local_user::LocalUser_, Crud};
  use lemmy_db_schema::source::{
    local_user::{LocalUser, LocalUserForm},
    person::{Person, PersonForm},
  };
  use lemmy_utils::claims::Claims;

  #[test]
  fn test_should_not_validate_user_token_after_password_change() {
    let conn = establish_unpooled_connection();

    let new_person = PersonForm {
      name: "Gerry9812".into(),
      preferred_username: None,
      avatar: None,
      banner: None,
      banned: None,
      deleted: None,
      published: None,
      updated: None,
      actor_id: None,
      bio: None,
      local: None,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
      inbox_url: None,
      shared_inbox_url: None,
    };

    let inserted_person = Person::create(&conn, &new_person).unwrap();

    let local_user_form = LocalUserForm {
      person_id: inserted_person.id,
      email: None,
      matrix_user_id: None,
      password_encrypted: "123456".to_string(),
      admin: None,
      show_nsfw: None,
      theme: None,
      default_sort_type: None,
      default_listing_type: None,
      lang: None,
      show_avatars: None,
      send_notifications_to_email: None,
    };

    let inserted_local_user = LocalUser::create(&conn, &local_user_form).unwrap();

    let jwt = Claims::jwt(inserted_local_user.id.0).unwrap();
    let claims = Claims::decode(&jwt).unwrap().claims;
    let check = check_validator_time(&inserted_local_user.validator_time, &claims);
    assert!(check.is_ok());

    // The check should fail, since the validator time is now newer than the jwt issue time
    let updated_local_user =
      LocalUser::update_password(&conn, inserted_local_user.id, &"password111").unwrap();
    let check_after = check_validator_time(&updated_local_user.validator_time, &claims);
    assert!(check_after.is_err());

    let num_deleted = Person::delete(&conn, inserted_person.id).unwrap();
    assert_eq!(1, num_deleted);
  }

  #[test]
  fn test_espeak() {
    assert!(captcha_espeak_wav_base64("WxRt2l").is_ok())
  }
}
