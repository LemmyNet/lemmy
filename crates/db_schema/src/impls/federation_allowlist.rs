use crate::{
  schema::federation_allowlist,
  source::{
    federation_allowlist::{FederationAllowList, FederationAllowListForm},
    instance::Instance,
  },
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

impl FederationAllowList {
  pub async fn replace(pool: &mut DbPool<'_>, list_opt: Option<Vec<String>>) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          if let Some(list) = list_opt {
            Self::clear(conn).await?;

            for domain in list {
              // Upsert all of these as instances
              let instance = Instance::read_or_create(&mut conn.into(), domain).await?;

              let form = FederationAllowListForm {
                instance_id: instance.id,
                updated: None,
              };
              insert_into(federation_allowlist::table)
                .values(form)
                .get_result::<Self>(conn)
                .await?;
            }
            Ok(())
          } else {
            Ok(())
          }
        }) as _
      })
      .await
  }

  async fn clear(conn: &mut AsyncPgConnection) -> Result<usize, Error> {
    diesel::delete(federation_allowlist::table)
      .execute(conn)
      .await
  }
}
#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{
    source::{federation_allowlist::FederationAllowList, instance::Instance},
    utils::build_db_pool_for_tests,
  };
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_allowlist_insert_and_clear() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let domains = vec![
      "tld1.xyz".to_string(),
      "tld2.xyz".to_string(),
      "tld3.xyz".to_string(),
    ];

    let allowed = Some(domains.clone());

    FederationAllowList::replace(pool, allowed).await.unwrap();

    let allows = Instance::allowlist(pool).await.unwrap();
    let allows_domains = allows
      .iter()
      .map(|i| i.domain.clone())
      .collect::<Vec<String>>();

    assert_eq!(3, allows.len());
    assert_eq!(domains, allows_domains);

    // Now test clearing them via Some(empty vec)
    let clear_allows = Some(Vec::new());

    FederationAllowList::replace(pool, clear_allows)
      .await
      .unwrap();
    let allows = Instance::allowlist(pool).await.unwrap();

    assert_eq!(0, allows.len());

    Instance::delete_all(pool).await.unwrap();
  }
}
