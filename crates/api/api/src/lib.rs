use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD as base64};
use captcha::Captcha;
use lemmy_api_utils::{context::LemmyContext, utils::is_mod_or_admin_opt};
use lemmy_db_schema::newtypes::CommunityId;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::slurs::check_slurs,
};
use regex::Regex;
use std::io::Cursor;
use totp_rs::{Secret, TOTP};

pub mod comment;
pub mod community;
pub mod federation;
pub mod local_user;
pub mod post;
pub mod reports;
pub mod site;
pub mod sitemap;

/// Converts the captcha to a base64 encoded wav audio file
pub(crate) fn captcha_as_wav_base64(captcha: &Captcha) -> LemmyResult<String> {
  let letters = captcha.as_wav();

  // Decode each wav file, concatenate the samples
  let mut concat_samples: Vec<i16> = Vec::new();
  let mut any_header: Option<hound::WavSpec> = None;
  for letter in letters {
    let mut cursor = Cursor::new(letter.unwrap_or_default());
    let reader = hound::WavReader::new(&mut cursor)?;
    any_header = Some(reader.spec());
    let samples16 = reader
      .into_samples::<i16>()
      .collect::<Result<Vec<_>, _>>()
      .with_lemmy_type(LemmyErrorType::CouldntCreateAudioCaptcha)?;
    concat_samples.extend(samples16);
  }

  // Encode the concatenated result as a wav file
  let mut output_buffer = Cursor::new(vec![]);
  if let Some(header) = any_header {
    let mut writer = hound::WavWriter::new(&mut output_buffer, header)
      .with_lemmy_type(LemmyErrorType::CouldntCreateAudioCaptcha)?;
    let mut writer16 = writer.get_i16_writer(concat_samples.len().try_into()?);
    for sample in concat_samples {
      writer16.write_sample(sample);
    }
    writer16
      .flush()
      .with_lemmy_type(LemmyErrorType::CouldntCreateAudioCaptcha)?;
    writer
      .finalize()
      .with_lemmy_type(LemmyErrorType::CouldntCreateAudioCaptcha)?;

    Ok(base64.encode(output_buffer.into_inner()))
  } else {
    Err(LemmyErrorType::CouldntCreateAudioCaptcha)?
  }
}

/// Check size of report
pub(crate) fn check_report_reason(reason: &str, slur_regex: &Regex) -> LemmyResult<()> {
  check_slurs(reason, slur_regex)?;
  if reason.is_empty() {
    Err(LemmyErrorType::ReportReasonRequired)?
  } else if reason.chars().count() > 1000 {
    Err(LemmyErrorType::ReportTooLong)?
  } else {
    Ok(())
  }
}

pub(crate) fn check_totp_2fa_valid(
  local_user_view: &LocalUserView,
  totp_token: &Option<String>,
  site_name: &str,
) -> LemmyResult<()> {
  // Throw an error if their token is missing
  let token = totp_token
    .as_deref()
    .ok_or(LemmyErrorType::MissingTotpToken)?;
  let secret = local_user_view
    .local_user
    .totp_2fa_secret
    .as_deref()
    .ok_or(LemmyErrorType::MissingTotpSecret)?;

  let totp = build_totp_2fa(site_name, &local_user_view.person.name, secret)?;

  let check_passed = totp.check_current(token)?;
  if !check_passed {
    return Err(LemmyErrorType::IncorrectTotpToken.into());
  }

  Ok(())
}

pub(crate) fn generate_totp_2fa_secret() -> String {
  Secret::generate_secret().to_string()
}

fn build_totp_2fa(hostname: &str, username: &str, secret: &str) -> LemmyResult<TOTP> {
  let sec = Secret::Raw(secret.as_bytes().to_vec());
  let sec_bytes = sec
    .to_bytes()
    .with_lemmy_type(LemmyErrorType::CouldntParseTotpSecret)?;

  TOTP::new(
    totp_rs::Algorithm::SHA1,
    6,
    1,
    30,
    sec_bytes,
    Some(hostname.to_string()),
    username.to_string(),
  )
  .with_lemmy_type(LemmyErrorType::CouldntGenerateTotp)
}

/// Only show the modlog names if:
/// You're an admin or
/// You're fetching the modlog for a single community, and you're a mod
/// (Alternatively !admin/mod)
async fn hide_modlog_names(
  local_user_view: Option<&LocalUserView>,
  community_id: Option<CommunityId>,
  context: &LemmyContext,
) -> bool {
  if let Some(community_id) = community_id {
    is_mod_or_admin_opt(&mut context.pool(), local_user_view, Some(community_id))
      .await
      .is_err()
  } else {
    !local_user_view
      .map(|l| l.local_user.admin)
      .unwrap_or_default()
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_build_totp() {
    let generated_secret = generate_totp_2fa_secret();
    let totp = build_totp_2fa("lemmy.ml", "my_name", &generated_secret);
    assert!(totp.is_ok());
  }
}
