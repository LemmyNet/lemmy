use crate::{
  schema::blocklist,
  source::{
    blocklist::{BlockList, BlockListForm},
    instance::{Instance, InstanceForm},
  },
};
use diesel::{dsl::*, result::Error, *};

impl BlockList {
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

            let form = BlockListForm {
              instance_id: instance.id,
              updated: None,
            };
            insert_into(blocklist::table)
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
    diesel::delete(blocklist::table).execute(conn)
  }
}
