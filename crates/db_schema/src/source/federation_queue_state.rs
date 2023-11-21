use crate::newtypes::{ActivityId, InstanceId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Insertable, AsChangeset)
)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", diesel(table_name = crate::schema::federation_queue_state))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct FederationQueueState {
  pub instance_id: InstanceId,
  /// the last successfully sent activity id
  pub last_successful_id: Option<ActivityId>,
  pub last_successful_published_time: Option<DateTime<Utc>>,
  /// how many failed attempts have been made to send the next activity
  pub fail_count: i32,
  /// timestamp of the last retry attempt (when the last failing activity was resent)
  pub last_retry: Option<DateTime<Utc>>,
}
