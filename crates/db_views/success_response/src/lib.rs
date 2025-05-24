use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

mod impls;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(ts_rs::TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response that completes successfully.
pub struct SuccessResponse {
  pub success: bool,
}
