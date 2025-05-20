use lemmy_db_schema::sensitive::SensitiveString;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Make a request to resend your verification email.
pub struct ResendVerificationEmail {
  pub email: SensitiveString,
}
