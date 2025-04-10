use crate::{
  newtypes::InstanceId,
  source::federation_blocklist::{FederationBlockList, FederationBlockListForm},
  utils::{get_conn, DbPool},
};
use diesel::{delete, dsl::insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::federation_blocklist;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl FederationBlockList {
  pub async fn block(pool: &mut DbPool<'_>, form: &FederationBlockListForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(federation_blocklist::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntBlockInstance)
  }
  pub async fn unblock(pool: &mut DbPool<'_>, instance_id_: InstanceId) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    delete(federation_blocklist::table.filter(federation_blocklist::instance_id.eq(instance_id_)))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::Deleted)
  }
}
