use crate::{
  newtypes::InstanceId,
  source::federation_blocklist::{FederationBlockList, FederationBlockListForm},
  utils::{get_conn, DbPool},
};
use diesel::{delete, dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::federation_blocklist;

impl FederationBlockList {
  pub async fn block(pool: &mut DbPool<'_>, form: &FederationBlockListForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(federation_blocklist::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn unblock(pool: &mut DbPool<'_>, instance_id_: InstanceId) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    delete(federation_blocklist::table.filter(federation_blocklist::instance_id.eq(instance_id_)))
      .execute(conn)
      .await
  }
}
