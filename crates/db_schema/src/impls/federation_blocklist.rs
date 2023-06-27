use crate::{
  newtypes::PersonId,
  schema::{admin_block_instance, federation_blocklist, instance},
  source::{
    federation_blocklist::{BlockInstanceAction, FederationBlockList, FederationBlockListForm},
    instance::Instance,
    moderator::{AdminBlockInstance, AdminBlockInstanceForm},
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::insert_into,
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use std::collections::{HashMap, HashSet};

impl FederationBlockList {
  pub async fn replace(
    pool: &DbPool,
    list_opt: Option<Vec<BlockInstanceAction>>,
    admin_person_id: PersonId,
  ) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          if let Some(list) = list_opt {
            let existing_blocked_instances: HashMap<String, Option<String>> =
              federation_blocklist::table
                .inner_join(instance::table)
                .inner_join(
                  admin_block_instance::table.on(
                    instance::id
                      .eq(admin_block_instance::instance_id)
                      .and(admin_block_instance::blocked),
                  ),
                )
                .order_by((instance::id, admin_block_instance::when_.desc()))
                .distinct_on(instance::id)
                .select((instance::domain, admin_block_instance::reason))
                .get_results::<(String, Option<String>)>(conn)
                .await?
                .into_iter()
                .collect::<HashMap<String, Option<String>>>();

            Self::clear(conn).await?;

            let mut added_or_updated_blocks = Vec::new();
            let mut unblocked_instances = existing_blocked_instances
              .keys()
              .cloned()
              .collect::<HashSet<String>>();

            for action in &list {
              // Upsert all of these as instances
              let instance =
                Instance::read_or_create_with_conn(conn, action.domain.clone()).await?;

              let form = FederationBlockListForm {
                instance_id: instance.id,
                updated: None,
              };
              insert_into(federation_blocklist::table)
                .values(form)
                .get_result::<Self>(conn)
                .await?;

              if let Some(maybe_existing_reason) = existing_blocked_instances.get(&action.domain) {
                if maybe_existing_reason != &action.reason {
                  // existing instance, with new reason
                  added_or_updated_blocks.push(AdminBlockInstanceForm {
                    admin_person_id,
                    instance_id: instance.id,
                    reason: action.reason.clone(),
                    blocked: true,
                  });
                }
              } else {
                // new instance blocked
                added_or_updated_blocks.push(AdminBlockInstanceForm {
                  admin_person_id,
                  instance_id: instance.id,
                  reason: action.reason.clone(),
                  blocked: true,
                });
              }
              unblocked_instances.remove(&action.domain);
            }

            for unblocked_domain in unblocked_instances {
              let instance = Instance::read_or_create_with_conn(conn, unblocked_domain).await?;

              // instance unblocked since the domain was not referenced in the new blocklist
              AdminBlockInstance::create(
                pool,
                &AdminBlockInstanceForm {
                  admin_person_id,
                  instance_id: instance.id,
                  reason: None,
                  blocked: false,
                },
              )
              .await?;
            }

            for added_or_updated_block in added_or_updated_blocks {
              AdminBlockInstance::create(pool, &added_or_updated_block).await?;
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
      person::{Person, PersonInsertForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_blocklist_replace_empty_with_populated() {
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

    let admin = PersonInsertForm::builder()
      .name("bob".into())
      .public_key("pubkey".into())
      .instance_id(instance_a.id)
      .admin(Some(true))
      .build();

    let admin_inserted = Person::create(pool, &admin).await.unwrap();

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

    let result =
      FederationBlockList::replace(pool, Some(instances_to_block), admin_inserted.id).await;

    assert!(result.is_ok());

    let updated_blocklist = Instance::blocklist(pool).await.unwrap();

    assert_eq!(updated_blocklist.len(), 3);

    assert_eq!(updated_blocklist[0].0, instance_a);
    assert_eq!(
      updated_blocklist[0].1.as_ref().unwrap().reason,
      Some("various reasons".to_string())
    );

    assert_eq!(updated_blocklist[1].0, instance_b);
    assert_eq!(updated_blocklist[1].1.as_ref().unwrap().reason, None);

    assert_eq!(updated_blocklist[2].0, instance_c);
    assert_eq!(
      updated_blocklist[2].1.as_ref().unwrap().reason,
      Some("even more reasons".to_string())
    );
  }

  #[tokio::test]
  #[serial]
  async fn test_blocklist_replace_populated_replaced_with_empty() {
    let pool = &build_db_pool_for_tests().await;
    let domain_a = "a.a".to_string();
    let domain_b = "b.b".to_string();
    let domain_c = "c.c".to_string();
    let domain_d = "d.d".to_string();

    let instance_a = Instance::read_or_create(pool, domain_a.clone())
      .await
      .unwrap();
    let _instance_b = Instance::read_or_create(pool, domain_b.clone())
      .await
      .unwrap();
    let _instance_c = Instance::read_or_create(pool, domain_c.clone())
      .await
      .unwrap();
    let _instance_d = Instance::read_or_create(pool, domain_d.clone())
      .await
      .unwrap();

    let admin = PersonInsertForm::builder()
      .name("bob".into())
      .public_key("pubkey".into())
      .instance_id(instance_a.id)
      .admin(Some(true))
      .build();

    let admin_inserted = Person::create(pool, &admin).await.unwrap();

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

    FederationBlockList::replace(pool, Some(instances_to_block), admin_inserted.id)
      .await
      .unwrap();

    let remove_all_blocks_result =
      FederationBlockList::replace(pool, Some(Vec::new()), admin_inserted.id).await;

    assert!(remove_all_blocks_result.is_ok());

    let updated_blocklist = Instance::blocklist(pool).await.unwrap();

    assert_eq!(updated_blocklist.len(), 0);
  }

  #[tokio::test]
  #[serial]
  async fn test_blocklist_replace_reasons() {
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

    let admin = PersonInsertForm::builder()
      .name("bob".into())
      .public_key("pubkey".into())
      .instance_id(instance_a.id)
      .admin(Some(true))
      .build();

    let admin_inserted = Person::create(pool, &admin).await.unwrap();

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

    FederationBlockList::replace(pool, Some(instances_to_block), admin_inserted.id)
      .await
      .unwrap();

    let new_reasons = Vec::from([
      BlockInstanceAction {
        domain: domain_a.clone(),
        reason: Some("various, new reasons".to_string()),
      },
      BlockInstanceAction {
        domain: domain_b.clone(),
        reason: Some("provided a reason now".to_string()),
      },
      BlockInstanceAction {
        domain: domain_c.clone(),
        reason: None,
      },
    ]);

    let updated_reasons_result =
      FederationBlockList::replace(pool, Some(new_reasons), admin_inserted.id).await;

    assert!(updated_reasons_result.is_ok());

    let updated_blocklist = Instance::blocklist(pool).await.unwrap();

    assert_eq!(updated_blocklist.len(), 3);

    assert_eq!(updated_blocklist[0].0, instance_a);
    assert_eq!(
      updated_blocklist[0].1.as_ref().unwrap().reason,
      Some("various, new reasons".to_string())
    );

    assert_eq!(updated_blocklist[1].0, instance_b);
    assert_eq!(
      updated_blocklist[1].1.as_ref().unwrap().reason,
      Some("provided a reason now".to_string())
    );

    assert_eq!(updated_blocklist[2].0, instance_c);
    assert_eq!(updated_blocklist[2].1.as_ref().unwrap().reason, None);
  }
}
