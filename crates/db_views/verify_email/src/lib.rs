use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Verify your email.
pub struct VerifyEmail {
  pub token: String,
}
