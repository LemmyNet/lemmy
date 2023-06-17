use crate::{
  schema::federation_blocklist,
  source::{
    federation_blocklist::{BlockInstanceAction, FederationBlockList, FederationBlockListForm},
    instance::Instance,
  },
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

impl FederationBlockList {
  pub async fn replace(
    pool: &DbPool,
    list_opt: Option<Vec<BlockInstanceAction>>,
  ) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          if let Some(list) = list_opt {
            Self::clear(conn).await?;

            for action in list {
              // Upsert all of these as instances
              let instance = Instance::read_or_create_with_conn(conn, action.domain).await?;

              let form = FederationBlockListForm {
                instance_id: instance.id,
                updated: None,
                reason: action.reason,
              };
              insert_into(federation_blocklist::table)
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
    diesel::delete(federation_blocklist::table)
      .execute(conn)
      .await
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{
      federation_blocklist::{BlockInstanceAction, FederationBlockList},
      instance::Instance,
    },
    utils::build_db_pool_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_blocklist_replace() {
    let pool = &build_db_pool_for_tests().await;
    let domain_a = "a.a".to_string();
    let domain_b = "b.b".to_string();
    let domain_c = "c.c".to_string();
    let domain_d = "d.d".to_string();

    let instance_a = Instance::read_or_create(pool, domain_a.clone())
      .await
      .unwrap();
    let instance_b = Instance::read_or_create(pool, domain_b.clone())
      .await
      .unwrap();
    let instance_c = Instance::read_or_create(pool, domain_c.clone())
      .await
      .unwrap();
    let _instance_d = Instance::read_or_create(pool, domain_d.clone())
      .await
      .unwrap();

    let initial_blocklist = Instance::blocklist(pool).await.unwrap();

    assert_eq!(initial_blocklist, Vec::default());

    let instances_to_block = Vec::from([
      BlockInstanceAction {
        domain: domain_a.clone(),
        reason: Some("various reasons".to_string()),
      },
      BlockInstanceAction {
        domain: domain_b.clone(),
        reason: None,
      },
      BlockInstanceAction {
        domain: domain_c.clone(),
        reason: Some("even more reasons".to_string()),
      },
    ]);

    let result = FederationBlockList::replace(pool, Some(instances_to_block)).await;

    assert!(result.is_ok());

    let updated_blocklist = Instance::blocklist(pool).await.unwrap();

    assert_eq!(updated_blocklist.len(), 3);

    assert_eq!(updated_blocklist[0].0, instance_a);
    assert_eq!(
      updated_blocklist[0].1.reason,
      Some("various reasons".to_string())
    );

    assert_eq!(updated_blocklist[1].0, instance_b);
    assert_eq!(updated_blocklist[1].1.reason, None);

    assert_eq!(updated_blocklist[2].0, instance_c);
    assert_eq!(
      updated_blocklist[2].1.reason,
      Some("even more reasons".to_string())
    );
  }
}
