use crate::newtypes::{ActivityId, InstanceId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Insertable, AsChangeset)
)]
#[cfg_attr(feature = "full", diesel(table_name = lemmy_db_schema_file::schema::federation_queue_state))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct FederationQueueState {
  pub instance_id: InstanceId,
  /// the last successfully sent activity id
  pub last_successful_id: Option<ActivityId>,
  pub last_successful_published_time_at: Option<DateTime<Utc>>,
  /// how many failed attempts have been made to send the next activity
  pub fail_count: i32,
  /// timestamp of the last retry attempt (when the last failing activity was resent)
  pub last_retry_at: Option<DateTime<Utc>>,
}
