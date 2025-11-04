use crate::{newtypes::InstanceId, source::federation_queue_state::FederationQueueState};
use diesel::{ExpressionMethods, Insertable, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::federation_queue_state;
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl FederationQueueState {
  /// load state or return a default empty value
  pub async fn load(pool: &mut DbPool<'_>, instance_id: InstanceId) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      federation_queue_state::table
        .filter(federation_queue_state::instance_id.eq(instance_id))
        .select(FederationQueueState::as_select())
        .get_result(conn)
        .await
        .optional()?
        .unwrap_or(FederationQueueState {
          instance_id,
          fail_count: 0,
          last_retry_at: None,
          last_successful_id: None, // this value is set to the most current id for new instances
          last_successful_published_time_at: None,
        }),
    )
  }
  pub async fn upsert(pool: &mut DbPool<'_>, state: &FederationQueueState) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;

    state
      .insert_into(federation_queue_state::table)
      .on_conflict(federation_queue_state::instance_id)
      .do_update()
      .set(state)
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}
