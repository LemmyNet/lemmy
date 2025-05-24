use lemmy_db_views_federated_instances::FederatedInstances;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response of federated instances.
pub struct GetFederatedInstancesResponse {
  /// Optional, because federation may be disabled.
  #[cfg_attr(feature = "full", ts(optional))]
  pub federated_instances: Option<FederatedInstances>,
}
