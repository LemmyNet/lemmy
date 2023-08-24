use actix_web::web::Data;
use base64::{engine::general_purpose::STANDARD_NO_PAD as base64, Engine};
use captcha::Captcha;
use lemmy_api_common::{context::LemmyContext, utils::local_site_to_slur_regex};
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::slurs::check_slurs,
};
use std::io::Cursor;

pub mod comment;
pub mod comment_report;
pub mod community;
pub mod local_user;
pub mod post;
pub mod post_report;
pub mod private_message;
pub mod private_message_report;
pub mod site;
pub mod sitemap;

#[async_trait::async_trait(?Send)]
pub trait Perform {
  type Response: serde::ser::Serialize + Send + Clone + Sync;

  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError>;
}

/// Converts the captcha to a base64 encoded wav audio file
pub(crate) fn captcha_as_wav_base64(captcha: &Captcha) -> Result<String, LemmyError> {
  let letters = captcha.as_wav();

  // Decode each wav file, concatenate the samples
  let mut concat_samples: Vec<i16> = Vec::new();
  let mut any_header: Option<wav::Header> = None;
  for letter in letters {
    let mut cursor = Cursor::new(letter.unwrap_or_default());
    let (header, samples) = wav::read(&mut cursor)?;
    any_header = Some(header);
    if let Some(samples16) = samples.as_sixteen() {
      concat_samples.extend(samples16);
    } else {
      Err(LemmyErrorType::CouldntCreateAudioCaptcha)?
    }
  }

  // Encode the concatenated result as a wav file
  let mut output_buffer = Cursor::new(vec![]);
  let header = match any_header {
    Some(header) => header,
    None => return Err(LemmyErrorType::CouldntCreateAudioCaptcha)?,
  };
  wav::write(
    header,
    &wav::BitDepth::Sixteen(concat_samples),
    &mut output_buffer,
  )
  .with_lemmy_type(LemmyErrorType::CouldntCreateAudioCaptcha)?;

  Ok(base64.encode(output_buffer.into_inner()))
}

/// Check size of report
pub(crate) fn check_report_reason(reason: &str, local_site: &LocalSite) -> Result<(), LemmyError> {
  let slur_regex = &local_site_to_slur_regex(local_site);

  check_slurs(reason, slur_regex)?;
  if reason.is_empty() {
    Err(LemmyErrorType::ReportReasonRequired)?
  } else if reason.chars().count() > 1000 {
    Err(LemmyErrorType::ReportTooLong)?
  } else {
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

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
    let pool = &mut pool.into();
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
