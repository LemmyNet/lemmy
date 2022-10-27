use crate::{
  schema::federation_blocklist,
  source::{
    federation_blocklist::{FederationBlockList, FederationBlockListForm},
    instance::Instance,
  },
};
use diesel::{dsl::*, result::Error, *};

impl FederationBlockList {
  pub fn replace(conn: &mut PgConnection, list_opt: Option<Vec<String>>) -> Result<(), Error> {
    conn.build_transaction().read_write().run(|conn| {
      if let Some(list) = list_opt {
        Self::clear(conn)?;

        for domain in list {
          // Upsert all of these as instances
          let instance = Instance::create(conn, &domain)?;

          let form = FederationBlockListForm {
            instance_id: instance.id,
            updated: None,
          };
          insert_into(federation_blocklist::table)
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
    diesel::delete(federation_blocklist::table).execute(conn)
  }
}
