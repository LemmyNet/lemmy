use crate::{
  newtypes::InstanceId,
  schema::{allowlist, blocklist, instance},
  source::instance::{Instance, InstanceForm},
};
use diesel::{dsl::*, result::Error, *};

impl Instance {
  pub fn create(conn: &mut PgConnection, form: &InstanceForm) -> Result<Self, Error> {
    // Do upsert on domain name conflict
    insert_into(instance::table)
      .values(form)
      .on_conflict(instance::domain)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
  }
  pub fn delete(conn: &mut PgConnection, instance_id: InstanceId) -> Result<usize, Error> {
    diesel::delete(instance::table.find(instance_id)).execute(conn)
  }
  pub fn delete_all(conn: &mut PgConnection) -> Result<usize, Error> {
    diesel::delete(instance::table).execute(conn)
  }
  pub fn allowlist(conn: &mut PgConnection) -> Result<Vec<String>, Error> {
    instance::table
      .inner_join(allowlist::table)
      .select(instance::domain)
      .load::<String>(conn)
  }

  pub fn blocklist(conn: &mut PgConnection) -> Result<Vec<String>, Error> {
    instance::table
      .inner_join(blocklist::table)
      .select(instance::domain)
      .load::<String>(conn)
  }

  pub fn linked(conn: &mut PgConnection) -> Result<Vec<String>, Error> {
    instance::table
      .left_join(blocklist::table)
      .filter(blocklist::id.is_null())
      .select(instance::domain)
      .load::<String>(conn)
  }
}
