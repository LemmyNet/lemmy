use chrono::{DateTime, Utc};
use lemmy_db_schema::source::federation_queue_state::FederationQueueState;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

mod impls;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ReadableFederationState {
  #[serde(flatten)]
  internal_state: FederationQueueState,
  /// timestamp of the next retry attempt (null if fail count is 0)
  #[cfg_attr(feature = "full", ts(optional))]
  next_retry: Option<DateTime<Utc>>,
}
