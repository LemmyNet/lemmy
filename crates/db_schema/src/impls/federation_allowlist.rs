use crate::source::federation_allowlist::{FederationAllowList, FederationAllowListForm};
use diesel::{ExpressionMethods, QueryDsl, delete, dsl::insert_into};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::{InstanceId, schema::federation_allowlist};
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl FederationAllowList {
  pub async fn allow(pool: &mut DbPool<'_>, form: &FederationAllowListForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(federation_allowlist::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }
  pub async fn unallow(pool: &mut DbPool<'_>, instance_id_: InstanceId) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    delete(federation_allowlist::table.filter(federation_allowlist::instance_id.eq(instance_id_)))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::Deleted)
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::source::instance::Instance;
  use lemmy_diesel_utils::connection::build_db_pool_for_tests;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_allowlist_insert_and_clear() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let instances = vec![
      Instance::read_or_create(pool, "tld1.xyz").await?,
      Instance::read_or_create(pool, "tld2.xyz").await?,
      Instance::read_or_create(pool, "tld3.xyz").await?,
    ];
    let forms: Vec<_> = instances
      .iter()
      .map(|i| FederationAllowListForm::new(i.id))
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
