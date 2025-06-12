use chrono::{DateTime, Utc};
use lemmy_db_schema::source::federation_queue_state::FederationQueueState;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

mod impls;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ReadableFederationState {
  #[serde(flatten)]
  internal_state: FederationQueueState,
  /// timestamp of the next retry attempt (null if fail count is 0)
  next_retry: Option<DateTime<Utc>>,
}
