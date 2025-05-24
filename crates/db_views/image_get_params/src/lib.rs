use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ImageGetParams {
  #[cfg_attr(feature = "full", ts(optional))]
  pub file_type: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub max_size: Option<i32>,
}
