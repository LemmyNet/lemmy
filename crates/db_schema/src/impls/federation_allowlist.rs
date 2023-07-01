use crate::{
  schema::federation_allowlist,
  source::{
    federation_allowlist::{FederationAllowList, FederationAllowListForm},
    instance::Instance,
  },
  utils::DbConn,
};
use diesel::{dsl::insert_into, result::Error};
use diesel_async::RunQueryDsl;

impl FederationAllowList {
  pub async fn replace(mut conn: impl DbConn, list_opt: Option<Vec<String>>) -> Result<(), Error> {
    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          if let Some(list) = list_opt {
            Self::clear(&mut *conn).await?;

            for domain in list {
              // Upsert all of these as instances
              let instance = Instance::read_or_create_with_conn(&mut *conn, domain).await?;

              let form = FederationAllowListForm {
                instance_id: instance.id,
                updated: None,
              };
              insert_into(federation_allowlist::table)
                .values(form)
                .get_result::<Self>(&mut *conn)
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

  async fn clear(mut conn: impl DbConn) -> Result<usize, Error> {
    diesel::delete(federation_allowlist::table)
      .execute(&mut *conn)
      .await
  }
}
#[cfg(test)]
mod tests {
  use crate::{
    source::{federation_allowlist::FederationAllowList, instance::Instance},
    utils::build_db_conn_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_allowlist_insert_and_clear() {
    let mut conn = build_db_conn_for_tests().await;
    let domains = vec![
      "tld1.xyz".to_string(),
      "tld2.xyz".to_string(),
      "tld3.xyz".to_string(),
    ];

    let allowed = Some(domains.clone());

    FederationAllowList::replace(&mut *conn, allowed)
      .await
      .unwrap();

    let allows = Instance::allowlist(&mut *conn).await.unwrap();
    let allows_domains = allows
      .iter()
      .map(|i| i.domain.clone())
      .collect::<Vec<String>>();

    assert_eq!(3, allows.len());
    assert_eq!(domains, allows_domains);

    // Now test clearing them via Some(empty vec)
    let clear_allows = Some(Vec::new());

    FederationAllowList::replace(&mut *conn, clear_allows)
      .await
      .unwrap();
    let allows = Instance::allowlist(&mut *conn).await.unwrap();

    assert_eq!(0, allows.len());

    Instance::delete_all(&mut *conn).await.unwrap();
  }
}
