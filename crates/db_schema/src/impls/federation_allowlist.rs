use crate::{
  newtypes::InstanceId,
  schema::{admin_allow_instance, federation_allowlist},
  source::federation_allowlist::{
    AdminAllowInstance,
    AdminAllowInstanceForm,
    FederationAllowListForm,
  },
  utils::{get_conn, DbPool},
};
use diesel::{delete, dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl AdminAllowInstance {
  pub async fn allow(pool: &mut DbPool<'_>, form: &AdminAllowInstanceForm) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          insert_into(admin_allow_instance::table)
            .values(form)
            .execute(conn)
            .await?;

          let form2 = FederationAllowListForm {
            instance_id: form.instance_id,
            updated: None,
          };
          insert_into(federation_allowlist::table)
            .values(form2)
            .execute(conn)
            .await?;
          Ok(())
        })
      })
      .await
  }
  pub async fn unallow(pool: &mut DbPool<'_>, instance_id: InstanceId) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    delete(
      federation_allowlist::table.filter(federation_allowlist::dsl::instance_id.eq(instance_id)),
    )
    .execute(conn)
    .await?;
    Ok(())
  }
}
