use crate::{
  newtypes::InstanceId,
  schema::{admin_block_instance, federation_allowlist, federation_blocklist, instance},
  source::{
    instance::{Instance, InstanceForm},
    moderator::AdminBlockInstance,
  },
  utils::{get_conn, naive_now, DbPool},
};
use diesel::{
  dsl::insert_into,
  result::Error,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

impl Instance {
  pub(crate) async fn read_or_create_with_conn(
    conn: &mut AsyncPgConnection,
    domain_: String,
  ) -> Result<Self, Error> {
    use crate::schema::instance::domain;
    // First try to read the instance row and return directly if found
    let instance = instance::table
      .filter(domain.eq(&domain_))
      .first::<Self>(conn)
      .await;
    match instance {
      Ok(i) => Ok(i),
      Err(diesel::NotFound) => {
        // Instance not in database yet, insert it
        let form = InstanceForm::builder()
          .domain(domain_)
          .updated(Some(naive_now()))
          .build();
        insert_into(instance::table)
          .values(&form)
          // Necessary because this method may be called concurrently for the same domain. This
          // could be handled with a transaction, but nested transactions arent allowed
          .on_conflict(instance::domain)
          .do_update()
          .set(&form)
          .get_result::<Self>(conn)
          .await
      }
      e => e,
    }
  }

  /// Attempt to read Instance column for the given domain. If it doesnt exist, insert a new one.
  /// There is no need for update as the domain of an existing instance cant change.
  pub async fn read_or_create(pool: &DbPool, domain: String) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    Self::read_or_create_with_conn(conn, domain).await
  }
  pub async fn delete(pool: &DbPool, instance_id: InstanceId) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(instance::table.find(instance_id))
      .execute(conn)
      .await
  }
  #[cfg(test)]
  pub async fn delete_all(pool: &DbPool) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(instance::table).execute(conn).await
  }
  pub async fn allowlist(pool: &DbPool) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance::table
      .inner_join(federation_allowlist::table)
      .select(instance::all_columns)
      .get_results(conn)
      .await
  }

  pub async fn blocklist(pool: &DbPool) -> Result<Vec<(Self, Option<AdminBlockInstance>)>, Error> {
    let conn = &mut get_conn(pool).await?;

    instance::table
      .inner_join(federation_blocklist::table)
      .left_join(admin_block_instance::table)
      .order_by((instance::id, admin_block_instance::when_.desc()))
      .distinct_on(instance::id)
      .select((
        instance::all_columns,
        admin_block_instance::all_columns.nullable(),
      ))
      .get_results(conn)
      .await
  }

  pub async fn linked(pool: &DbPool) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance::table
      .left_join(federation_blocklist::table)
      .filter(federation_blocklist::id.is_null())
      .select(instance::all_columns)
      .get_results(conn)
      .await
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    schema::federation_blocklist,
    source::instance::Instance,
    utils::{build_db_pool_for_tests, get_conn},
  };
  use diesel::ExpressionMethods;
  use diesel_async::RunQueryDsl;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn backwards_compatiblity_test_fetching_blocklist_no_admin_log() {
    let pool = &build_db_pool_for_tests().await;

    let blocked_instance = Instance::read_or_create(pool, "abc.def".to_string())
      .await
      .unwrap();

    let conn = &mut get_conn(pool).await.unwrap();

    // clear federation blocklist in case of other tests having written to it
    diesel::delete(federation_blocklist::table)
      .execute(conn)
      .await
      .unwrap();

    // insert row into federation_blocklist without corresponding admin log reason, this represents a freshly migrated database
    diesel::insert_into(federation_blocklist::table)
      .values(federation_blocklist::instance_id.eq(blocked_instance.id))
      .execute(conn)
      .await
      .unwrap();

    let blocklist = Instance::blocklist(pool).await.unwrap();

    assert_eq!(blocklist.len(), 1);

    assert_eq!(blocklist[0].0, blocked_instance);
    assert_eq!(blocklist[0].1, None);
  }
}
