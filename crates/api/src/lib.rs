use actix_web::web::Data;
use captcha::Captcha;
use lemmy_api_common::{context::LemmyContext, utils::local_site_to_slur_regex};
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_utils::{error::LemmyError, utils::check_slurs, ConnectionId};

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
pub(crate) fn check_report_reason(reason: &str, local_site: &LocalSite) -> Result<(), LemmyError> {
  let slur_regex = &local_site_to_slur_regex(local_site);

  check_slurs(reason, slur_regex)?;
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
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      secret::Secret,
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use lemmy_utils::{claims::Claims, settings::SETTINGS};
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_should_not_validate_user_token_after_password_change() {
    let pool = &build_db_pool_for_tests().await;
    let secret = Secret::init(pool).await.unwrap();
    let settings = &SETTINGS.to_owned();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::builder()
      .name("Gerry9812".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_person.id)
      .password_encrypted("123456".to_string())
      .build();

    let inserted_local_user = LocalUser::create(pool, &local_user_form).await.unwrap();

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
      LocalUser::update_password(pool, inserted_local_user.id, "password111")
        .await
        .unwrap();
    let check_after = check_validator_time(&updated_local_user.validator_time, &claims);
    assert!(check_after.is_err());

    let num_deleted = Person::delete(pool, inserted_person.id).await.unwrap();
    assert_eq!(1, num_deleted);
  }
}
