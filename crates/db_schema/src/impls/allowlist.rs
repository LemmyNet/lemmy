use crate::{
  schema::allowlist,
  source::{
    allowlist::{AllowList, AllowListForm},
    instance::{Instance, InstanceForm},
  },
};
use diesel::{dsl::*, result::Error, *};

impl AllowList {
  pub fn replace(
    conn: &mut PgConnection,
    list_opt_str: Option<Option<String>>,
  ) -> Result<(), Error> {
    conn.build_transaction().read_write().run(|conn| {
      if let Some(list_str) = list_opt_str {
        Self::clear(conn)?;

        if let Some(list_replace_str) = list_str {
          let remove_whitespace = list_replace_str.split_whitespace().collect::<String>();
          let list = remove_whitespace.split(',').collect::<Vec<&str>>();
          for domain in list {
            // Upsert all of these as instances
            let instance_form = InstanceForm {
              domain: domain.to_string(),
              updated: None,
            };
            let instance = Instance::create(conn, &instance_form)?;

            let form = AllowListForm {
              instance_id: instance.id,
              updated: None,
            };
            insert_into(allowlist::table)
              .values(form)
              .get_result::<Self>(conn)?;
          }
          Ok(())
        } else {
          Ok(())
        }
      } else {
        Ok(())
      }
    })
  }

  pub fn clear(conn: &mut PgConnection) -> Result<usize, Error> {
    diesel::delete(allowlist::table).execute(conn)
  }
}
#[cfg(test)]
mod tests {
  use crate::{
    source::{allowlist::AllowList, instance::Instance},
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_allowlist_insert_and_clear() {
    let conn = &mut establish_unpooled_connection();
    let allowed = Some(Some("tld1.xyz, tld2.xyz,tld3.xyz".to_string()));

    AllowList::replace(conn, allowed).unwrap();

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

    // Now test clearing them via Some(none)
    let clear_allows = Some(None);

    AllowList::replace(conn, clear_allows).unwrap();
    let allows = Instance::allowlist(conn).unwrap();

    assert_eq!(0, allows.len());

    Instance::delete_all(conn).unwrap();
  }
}
