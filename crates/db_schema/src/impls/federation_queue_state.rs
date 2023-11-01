use crate::{
  newtypes::InstanceId,
  source::federation_queue_state::FederationQueueState,
  utils::{get_conn, DbPool},
};
use diesel::{prelude::*, result::Error};
use diesel_async::RunQueryDsl;

impl FederationQueueState {
  /// load state or return a default empty value
  pub async fn load(
    pool: &mut DbPool<'_>,
    instance_id_: InstanceId,
  ) -> Result<FederationQueueState, Error> {
    use crate::schema::federation_queue_state::dsl::{federation_queue_state, instance_id};
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
          last_retry: None,
          last_successful_id: None, // this value is set to the most current id for new instances
          last_successful_published_time: None,
        }),
    )
  }
  pub async fn upsert(pool: &mut DbPool<'_>, state: &FederationQueueState) -> Result<(), Error> {
    use crate::schema::federation_queue_state::dsl::{federation_queue_state, instance_id};
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
