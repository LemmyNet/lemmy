use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Does an apub fetch for an object.
pub struct ResolveObject {
  /// Can be the full url, or a shortened version like: !fediverse@lemmy.ml
  pub q: String,
}
