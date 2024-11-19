use crate::{
  schema::federation_blocklist,
  source::federation_blocklist::{FederationBlockList, FederationBlockListForm},
  utils::{get_conn, DbPool},
};
use diesel::{delete, ExpressionMethods, QueryDsl};
use diesel::{dsl::insert_into, result::Error};
use diesel_async::RunQueryDsl;

impl FederationBlockList {
  pub async fn block(pool: &mut DbPool<'_>, form: &FederationBlockListForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(federation_blocklist::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn unblock(
    pool: &mut DbPool<'_>,
    form: &FederationBlockListForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    delete(
      federation_blocklist::table
        .filter(federation_blocklist::dsl::instance_id.eq(form.instance_id)),
    )
    .get_result(conn)
    .await
  }
}
