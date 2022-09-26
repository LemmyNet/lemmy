use actix_web::{web, web::Data};
use captcha::Captcha;
use lemmy_api_common::{
  comment::*,
  community::*,
  person::*,
  post::*,
  private_message::*,
  site::*,
  websocket::*,
};
use lemmy_utils::{error::LemmyError, utils::check_slurs, ConnectionId};
use lemmy_websocket::{serialize_websocket_message, LemmyContext, UserOperation};
use serde::Deserialize;

mod comment;
mod comment_report;
mod community;
mod local_user;
mod post;
mod post_report;
mod private_message;
mod private_message_report;
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
  match op {
    // User ops
    UserOperation::Login => do_websocket_operation::<Login>(context, id, op, data).await,
    UserOperation::GetCaptcha => do_websocket_operation::<GetCaptcha>(context, id, op, data).await,
    UserOperation::GetReplies => do_websocket_operation::<GetReplies>(context, id, op, data).await,
    UserOperation::AddAdmin => do_websocket_operation::<AddAdmin>(context, id, op, data).await,
    UserOperation::GetUnreadRegistrationApplicationCount => {
      do_websocket_operation::<GetUnreadRegistrationApplicationCount>(context, id, op, data).await
    }
    UserOperation::ListRegistrationApplications => {
      do_websocket_operation::<ListRegistrationApplications>(context, id, op, data).await
    }
    UserOperation::ApproveRegistrationApplication => {
      do_websocket_operation::<ApproveRegistrationApplication>(context, id, op, data).await
    }
    UserOperation::BanPerson => do_websocket_operation::<BanPerson>(context, id, op, data).await,
    UserOperation::GetBannedPersons => {
      do_websocket_operation::<GetBannedPersons>(context, id, op, data).await
    }
    UserOperation::BlockPerson => {
      do_websocket_operation::<BlockPerson>(context, id, op, data).await
    }
    UserOperation::GetPersonMentions => {
      do_websocket_operation::<GetPersonMentions>(context, id, op, data).await
    }
    UserOperation::MarkPersonMentionAsRead => {
      do_websocket_operation::<MarkPersonMentionAsRead>(context, id, op, data).await
    }
    UserOperation::MarkCommentReplyAsRead => {
      do_websocket_operation::<MarkCommentReplyAsRead>(context, id, op, data).await
    }
    UserOperation::MarkAllAsRead => {
      do_websocket_operation::<MarkAllAsRead>(context, id, op, data).await
    }
    UserOperation::PasswordReset => {
      do_websocket_operation::<PasswordReset>(context, id, op, data).await
    }
    UserOperation::PasswordChange => {
      do_websocket_operation::<PasswordChangeAfterReset>(context, id, op, data).await
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
    UserOperation::ChangePassword => {
      do_websocket_operation::<ChangePassword>(context, id, op, data).await
    }
    UserOperation::GetReportCount => {
      do_websocket_operation::<GetReportCount>(context, id, op, data).await
    }
    UserOperation::GetUnreadCount => {
      do_websocket_operation::<GetUnreadCount>(context, id, op, data).await
    }
    UserOperation::VerifyEmail => {
      do_websocket_operation::<VerifyEmail>(context, id, op, data).await
    }

    // Private Message ops
    UserOperation::MarkPrivateMessageAsRead => {
      do_websocket_operation::<MarkPrivateMessageAsRead>(context, id, op, data).await
    }
    UserOperation::CreatePrivateMessageReport => {
      do_websocket_operation::<CreatePrivateMessageReport>(context, id, op, data).await
    }
    UserOperation::ResolvePrivateMessageReport => {
      do_websocket_operation::<ResolvePrivateMessageReport>(context, id, op, data).await
    }
    UserOperation::ListPrivateMessageReports => {
      do_websocket_operation::<ListPrivateMessageReports>(context, id, op, data).await
    }

    // Site ops
    UserOperation::GetModlog => do_websocket_operation::<GetModlog>(context, id, op, data).await,
    UserOperation::PurgePerson => {
      do_websocket_operation::<PurgePerson>(context, id, op, data).await
    }
    UserOperation::PurgeCommunity => {
      do_websocket_operation::<PurgeCommunity>(context, id, op, data).await
    }
    UserOperation::PurgePost => do_websocket_operation::<PurgePost>(context, id, op, data).await,
    UserOperation::PurgeComment => {
      do_websocket_operation::<PurgeComment>(context, id, op, data).await
    }
    UserOperation::Search => do_websocket_operation::<Search>(context, id, op, data).await,
    UserOperation::ResolveObject => {
      do_websocket_operation::<ResolveObject>(context, id, op, data).await
    }
    UserOperation::TransferCommunity => {
      do_websocket_operation::<TransferCommunity>(context, id, op, data).await
    }
    UserOperation::LeaveAdmin => do_websocket_operation::<LeaveAdmin>(context, id, op, data).await,

    // Community ops
    UserOperation::FollowCommunity => {
      do_websocket_operation::<FollowCommunity>(context, id, op, data).await
    }
    UserOperation::BlockCommunity => {
      do_websocket_operation::<BlockCommunity>(context, id, op, data).await
    }
    UserOperation::BanFromCommunity => {
      do_websocket_operation::<BanFromCommunity>(context, id, op, data).await
    }
    UserOperation::AddModToCommunity => {
      do_websocket_operation::<AddModToCommunity>(context, id, op, data).await
    }

    // Post ops
    UserOperation::LockPost => do_websocket_operation::<LockPost>(context, id, op, data).await,
    UserOperation::StickyPost => do_websocket_operation::<StickyPost>(context, id, op, data).await,
    UserOperation::CreatePostLike => {
      do_websocket_operation::<CreatePostLike>(context, id, op, data).await
    }
    UserOperation::MarkPostAsRead => {
      do_websocket_operation::<MarkPostAsRead>(context, id, op, data).await
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
    UserOperation::GetSiteMetadata => {
      do_websocket_operation::<GetSiteMetadata>(context, id, op, data).await
    }

    // Comment ops
    UserOperation::SaveComment => {
      do_websocket_operation::<SaveComment>(context, id, op, data).await
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
  let parsed_data: Data = serde_json::from_str(data)?;
  let res = parsed_data
    .perform(&web::Data::new(context), Some(id))
    .await?;
  serialize_websocket_message(&op, &res)
}

/// Converts the captcha to a base64 encoded wav audio file
pub(crate) fn captcha_as_wav_base64(captcha: &Captcha) -> String {
  let letters = captcha.as_wav();

  let mut concat_letters: Vec<u8> = Vec::new();

  for letter in letters {
    let bytes = letter.unwrap_or_default();
    concat_letters.extend(bytes);
  }

  // Convert to base64
  base64::encode(concat_letters)
}

/// Check size of report and remove whitespace
pub(crate) fn check_report_reason(reason: &str, context: &LemmyContext) -> Result<(), LemmyError> {
  check_slurs(reason, &context.settings().slur_regex())?;
  if reason.is_empty() {
    return Err(LemmyError::from_message("report_reason_required"));
  }
  if reason.chars().count() > 1000 {
    return Err(LemmyError::from_message("report_too_long"));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use lemmy_api_common::utils::check_validator_time;
  use lemmy_db_schema::{
    source::{
      local_user::{LocalUser, LocalUserForm},
      person::{Person, PersonForm},
      secret::Secret,
    },
    traits::Crud,
    utils::establish_unpooled_connection,
  };
  use lemmy_utils::{claims::Claims, settings::SETTINGS};

  #[test]
  fn test_should_not_validate_user_token_after_password_change() {
    let conn = &mut establish_unpooled_connection();
    let secret = Secret::init(conn).unwrap();
    let settings = &SETTINGS.to_owned();

    let new_person = PersonForm {
      name: "Gerry9812".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_person = Person::create(conn, &new_person).unwrap();

    let local_user_form = LocalUserForm {
      person_id: Some(inserted_person.id),
      password_encrypted: Some("123456".to_string()),
      ..LocalUserForm::default()
    };

    let inserted_local_user = LocalUser::create(conn, &local_user_form).unwrap();

    let jwt = Claims::jwt(
      inserted_local_user.id.0,
      &secret.jwt_secret,
      &settings.hostname,
    )
    .unwrap();
    let claims = Claims::decode(&jwt, &secret.jwt_secret).unwrap().claims;
    let check = check_validator_time(&inserted_local_user.validator_time, &claims);
    assert!(check.is_ok());

    // The check should fail, since the validator time is now newer than the jwt issue time
    let updated_local_user =
      LocalUser::update_password(conn, inserted_local_user.id, "password111").unwrap();
    let check_after = check_validator_time(&updated_local_user.validator_time, &claims);
    assert!(check_after.is_err());

    let num_deleted = Person::delete(conn, inserted_person.id).unwrap();
    assert_eq!(1, num_deleted);
  }
}
