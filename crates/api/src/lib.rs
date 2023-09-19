use base64::{engine::general_purpose::STANDARD_NO_PAD as base64, Engine};
use captcha::Captcha;
use lemmy_api_common::utils::{AUTH_COOKIE_NAME, local_site_to_slur_regex};
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::slurs::check_slurs,
};
use std::io::Cursor;
use actix_web::cookie::SameSite;
use actix_web::HttpRequest;

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
  if let Some(header) = any_header {
    wav::write(
      header,
      &wav::BitDepth::Sixteen(concat_samples),
      &mut output_buffer,
    )
    .with_lemmy_type(LemmyErrorType::CouldntCreateAudioCaptcha)?;

    Ok(base64.encode(output_buffer.into_inner()))
  } else {
    Err(LemmyErrorType::CouldntCreateAudioCaptcha)?
  }
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

pub fn read_auth_token(req: &HttpRequest) -> Result<Option<String>, LemmyError> {
// Try reading jwt from auth header
  let auth_header = req
      .headers()
      .get(AUTH_COOKIE_NAME)
      .and_then(|h| h.to_str().ok());
  let jwt = if let Some(a) = auth_header {
    Some(a.to_string())
  }
  // If that fails, try auth cookie. Dont use the `jwt` cookie from lemmy-ui because
  // its not http-only.
  else {
    let auth_cookie = req.cookie(AUTH_COOKIE_NAME);
    if let Some(a) = &auth_cookie {
      // ensure that its marked as httponly and secure
      let secure = a.secure().unwrap_or_default();
      let http_only = a.http_only().unwrap_or_default();
      let same_site = a.same_site();
      if !secure || !http_only || same_site != Some(SameSite::Strict) {
        return Err(LemmyError::from(LemmyErrorType::AuthCookieInsecure).into());
      }
    }
    auth_cookie.map(|c| c.value().to_string())
  };
  Ok(jwt)
}
