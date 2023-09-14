use crate::util::ActivityId;
use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::InstanceId,
  utils::{get_conn, DbPool},
};

#[derive(Queryable, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = lemmy_db_schema::schema::federation_queue_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FederationQueueState {
  pub instance_id: InstanceId,
  pub last_successful_id: ActivityId, // todo: i64
  pub fail_count: i32,
  pub last_retry: DateTime<Utc>,
}

impl FederationQueueState {
  /// load state or return a default empty value
  pub async fn load(
    pool: &mut DbPool<'_>,
    instance_id_: InstanceId,
  ) -> Result<FederationQueueState> {
    use lemmy_db_schema::schema::federation_queue_state::dsl::{
      federation_queue_state,
      instance_id,
    };
    let conn = &mut get_conn(pool).await?;
    Ok(
      federation_queue_state
        .filter(instance_id.eq(&instance_id_))
        .select(FederationQueueState::as_select())
        .get_result(conn)
        .await
        .optional()?
        .unwrap_or(FederationQueueState {
          instance_id: instance_id_,
          fail_count: 0,
          last_retry: Utc.timestamp_nanos(0),
          last_successful_id: -1, // this value is set to the most current id for new instances
        }),
    )
  }
  pub async fn upsert(pool: &mut DbPool<'_>, state: &FederationQueueState) -> Result<()> {
    use lemmy_db_schema::schema::federation_queue_state::dsl::{
      federation_queue_state,
      instance_id,
    };
    let conn = &mut get_conn(pool).await?;

    state
      .insert_into(federation_queue_state)
      .on_conflict(instance_id)
      .do_update()
      .set(state)
      .execute(conn)
      .await?;
    Ok(())
  }
}
