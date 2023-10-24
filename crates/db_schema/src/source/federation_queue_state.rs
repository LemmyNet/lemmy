use crate::newtypes::{ActivityId, InstanceId};
use chrono::{DateTime, Utc};
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::federation_queue_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FederationQueueState {
  pub instance_id: InstanceId,
  pub last_successful_id: ActivityId, // todo: i64
  pub fail_count: i32,
  pub last_retry: DateTime<Utc>,
}
