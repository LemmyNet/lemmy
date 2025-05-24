use lemmy_db_views_instance_with_federation_state::InstanceWithFederationState;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A list of federated instances.
pub struct FederatedInstances {
  pub linked: Vec<InstanceWithFederationState>,
  pub allowed: Vec<InstanceWithFederationState>,
  pub blocked: Vec<InstanceWithFederationState>,
}
