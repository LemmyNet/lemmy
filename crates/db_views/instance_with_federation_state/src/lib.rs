use lemmy_db_schema::source::instance::Instance;
use lemmy_db_views_readable_federation_state::ReadableFederationState;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct InstanceWithFederationState {
  #[serde(flatten)]
  pub instance: Instance,
  /// if federation to this instance is or was active, show state of outgoing federation to this
  /// instance
  #[cfg_attr(feature = "full", ts(optional))]
  pub federation_state: Option<ReadableFederationState>,
}
