use lemmy_db_schema::sensitive::SensitiveString;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Logging into lemmy.
///
/// Note: Banned users can still log in, to be able to do certain things like delete
/// their account.
pub struct Login {
  pub username_or_email: SensitiveString,
  pub password: SensitiveString,
  /// May be required, if totp is enabled for their account.
  #[cfg_attr(feature = "full", ts(optional))]
  pub totp_2fa_token: Option<String>,
}
