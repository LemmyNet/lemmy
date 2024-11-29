use crate::{
  newtypes::InstanceId,
  schema::{admin_allow_instance, federation_allowlist},
  source::{
    federation_allowlist::{FederationAllowList, FederationAllowListForm},
    mod_log::admin::{AdminAllowInstance, AdminAllowInstanceForm},
  },
  utils::{get_conn, DbPool},
};
use diesel::{delete, dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl AdminAllowInstance {
  pub async fn insert(pool: &mut DbPool<'_>, form: &AdminAllowInstanceForm) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(admin_allow_instance::table)
      .values(form)
      .execute(conn)
      .await?;

    Ok(())
  }
}

impl FederationAllowList {
  pub async fn allow(pool: &mut DbPool<'_>, form: &FederationAllowListForm) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(federation_allowlist::table)
      .values(form)
      .execute(conn)
      .await?;
    Ok(())
  }
  pub async fn unallow(pool: &mut DbPool<'_>, instance_id_: InstanceId) -> Result<(), Error> {
    use federation_allowlist::dsl::instance_id;
    let conn = &mut get_conn(pool).await?;
    delete(federation_allowlist::table.filter(instance_id.eq(instance_id_)))
      .execute(conn)
      .await?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::{source::instance::Instance, utils::build_db_pool_for_tests};
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_allowlist_insert_and_clear() -> Result<(), Error> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let instances = vec![
      Instance::read_or_create(pool, "tld1.xyz".to_string()).await?,
      Instance::read_or_create(pool, "tld2.xyz".to_string()).await?,
      Instance::read_or_create(pool, "tld3.xyz".to_string()).await?,
    ];
    let forms: Vec<_> = instances
      .iter()
      .map(|i| FederationAllowListForm {
        instance_id: i.id,
        updated: None,
      })
      .collect();

    for f in &forms {
      FederationAllowList::allow(pool, f).await?;
    }

    let allows = Instance::allowlist(pool).await?;

    assert_eq!(3, allows.len());
    assert_eq!(instances, allows);

    // Now test clearing them
    for f in forms {
      FederationAllowList::unallow(pool, f.instance_id).await?;
    }
    let allows = Instance::allowlist(pool).await?;
    assert_eq!(0, allows.len());

    Instance::delete_all(pool).await?;

    Ok(())
  }
}
