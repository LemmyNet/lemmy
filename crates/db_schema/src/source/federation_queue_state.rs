use crate::newtypes::{ActivityId, InstanceId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::prelude::*;

#[derive(Clone)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Insertable, AsChangeset)
)]
#[cfg_attr(feature = "full", diesel(table_name = crate::schema::federation_queue_state))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct FederationQueueState {
  pub instance_id: InstanceId,
  pub last_successful_id: ActivityId, // todo: i64
  pub fail_count: i32,
  pub last_retry: DateTime<Utc>,
}
