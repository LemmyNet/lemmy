use lemmy_db_schema::sensitive::SensitiveString;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Register / Sign up to lemmy.
pub struct Register {
  pub username: String,
  pub password: SensitiveString,
  pub password_verify: SensitiveString,
  #[cfg_attr(feature = "full", ts(optional))]
  pub show_nsfw: Option<bool>,
  /// email is mandatory if email verification is enabled on the server
  #[cfg_attr(feature = "full", ts(optional))]
  pub email: Option<SensitiveString>,
  /// The UUID of the captcha item.
  #[cfg_attr(feature = "full", ts(optional))]
  pub captcha_uuid: Option<String>,
  /// Your captcha answer.
  #[cfg_attr(feature = "full", ts(optional))]
  pub captcha_answer: Option<String>,
  /// A form field to trick signup bots. Should be None.
  #[cfg_attr(feature = "full", ts(optional))]
  pub honeypot: Option<String>,
  /// An answer is mandatory if require application is enabled on the server
  #[cfg_attr(feature = "full", ts(optional))]
  pub answer: Option<String>,
}
