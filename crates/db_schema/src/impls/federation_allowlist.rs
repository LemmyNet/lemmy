use crate::{
  schema::federation_allowlist,
  source::{
    federation_allowlist::{FederationAllowList, FederationAllowListForm},
    instance::Instance,
  },
};
use diesel::{dsl::*, result::Error, *};

impl FederationAllowList {
  pub fn replace(conn: &mut PgConnection, list_opt: Option<Vec<String>>) -> Result<(), Error> {
    conn.build_transaction().read_write().run(|conn| {
      if let Some(list) = list_opt {
        Self::clear(conn)?;

        for domain in list {
          // Upsert all of these as instances
          let instance = Instance::create(conn, &domain)?;

          let form = FederationAllowListForm {
            instance_id: instance.id,
            updated: None,
          };
          insert_into(federation_allowlist::table)
            .values(form)
            .get_result::<Self>(conn)?;
        }
        Ok(())
      } else {
        Ok(())
      }
    })
  }

  pub fn clear(conn: &mut PgConnection) -> Result<usize, Error> {
    diesel::delete(federation_allowlist::table).execute(conn)
  }
}
#[cfg(test)]
mod tests {
  use crate::{
    source::{federation_allowlist::FederationAllowList, instance::Instance},
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_allowlist_insert_and_clear() {
    let conn = &mut establish_unpooled_connection();
    let allowed = Some(vec![
      "tld1.xyz".to_string(),
      "tld2.xyz".to_string(),
      "tld3.xyz".to_string(),
    ]);

    FederationAllowList::replace(conn, allowed).unwrap();

    let allows = Instance::allowlist(conn).unwrap();

    assert_eq!(3, allows.len());
    assert_eq!(
      vec![
        "tld1.xyz".to_string(),
        "tld2.xyz".to_string(),
        "tld3.xyz".to_string()
      ],
      allows
    );

    // Now test clearing them via Some(empty vec)
    let clear_allows = Some(Vec::new());

    FederationAllowList::replace(conn, clear_allows).unwrap();
    let allows = Instance::allowlist(conn).unwrap();

    assert_eq!(0, allows.len());

    Instance::delete_all(conn).unwrap();
  }
}
