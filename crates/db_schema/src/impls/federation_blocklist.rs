use crate::{
  newtypes::InstanceId,
  schema::{admin_block_instance, federation_blocklist},
  source::{
    federation_blocklist::{FederationBlockList, FederationBlockListForm},
    mod_log::admin::{AdminBlockInstance, AdminBlockInstanceForm},
  },
  utils::{get_conn, DbPool},
};
use diesel::{delete, dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl AdminBlockInstance {
  pub async fn insert(pool: &mut DbPool<'_>, form: &AdminBlockInstanceForm) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_block_instance::table)
      .values(form)
      .execute(conn)
      .await?;

    Ok(())
  }
}

impl FederationBlockList {
  pub async fn block(pool: &mut DbPool<'_>, form: &FederationBlockListForm) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(federation_blocklist::table)
      .values(form)
      .execute(conn)
      .await?;
    Ok(())
  }
  pub async fn unblock(pool: &mut DbPool<'_>, instance_id_: InstanceId) -> Result<(), Error> {
    use federation_blocklist::dsl::instance_id;
    let conn = &mut get_conn(pool).await?;
    delete(federation_blocklist::table.filter(instance_id.eq(instance_id_)))
      .execute(conn)
      .await?;
    Ok(())
  }
}
