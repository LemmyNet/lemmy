use lemmy_db_views_captcha_response::CaptchaResponse;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A wrapper for the captcha response.
pub struct GetCaptchaResponse {
  /// Will be None if captchas are disabled.
  #[cfg_attr(feature = "full", ts(optional))]
  pub ok: Option<CaptchaResponse>,
}
