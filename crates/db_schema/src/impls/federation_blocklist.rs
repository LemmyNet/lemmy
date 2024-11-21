use crate::{
  newtypes::InstanceId,
  schema::{admin_block_instance, federation_blocklist},
  source::federation_blocklist::{
    AdminBlockInstance,
    AdminBlockInstanceForm,
    FederationBlockListForm,
  },
  utils::{get_conn, DbPool},
};
use diesel::{delete, dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl AdminBlockInstance {
  pub async fn block(pool: &mut DbPool<'_>, form: &AdminBlockInstanceForm) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          insert_into(admin_block_instance::table)
            .values(form)
            .execute(conn)
            .await?;

          let form2 = FederationBlockListForm {
            instance_id: form.instance_id,
            updated: None,
            expires: form.expires,
          };
          insert_into(federation_blocklist::table)
            .values(form2)
            .execute(conn)
            .await?;
          Ok(())
        })
      })
      .await
  }
  pub async fn unblock(pool: &mut DbPool<'_>, instance_id: InstanceId) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    delete(
      federation_blocklist::table.filter(federation_blocklist::dsl::instance_id.eq(instance_id)),
    )
    .execute(conn)
    .await?;
    Ok(())
  }
}
