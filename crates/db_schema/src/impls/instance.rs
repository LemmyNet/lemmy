use crate::{
  newtypes::InstanceId,
  schema::{federation_allowlist, federation_blocklist, instance},
  source::instance::{Instance, InstanceForm},
  utils::naive_now,
};
use diesel::{dsl::*, result::Error, *};
use lemmy_utils::utils::generate_domain_url;
use url::Url;

impl Instance {
  fn create_from_form(conn: &mut PgConnection, form: &InstanceForm) -> Result<Self, Error> {
    // Do upsert on domain name conflict
    insert_into(instance::table)
      .values(form)
      .on_conflict(instance::domain)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
  }
  pub fn create(conn: &mut PgConnection, domain: &str) -> Result<Self, Error> {
    let form = InstanceForm {
      domain: domain.to_string(),
      updated: Some(naive_now()),
    };
    Self::create_from_form(conn, &form)
  }
  pub fn create_from_actor_id(conn: &mut PgConnection, actor_id: &Url) -> Result<Self, Error> {
    let domain = &generate_domain_url(actor_id).expect("actor id missing a domain");
    Self::create(conn, domain)
  }
  pub fn delete(conn: &mut PgConnection, instance_id: InstanceId) -> Result<usize, Error> {
    diesel::delete(instance::table.find(instance_id)).execute(conn)
  }
  pub fn delete_all(conn: &mut PgConnection) -> Result<usize, Error> {
    diesel::delete(instance::table).execute(conn)
  }
  pub fn allowlist(conn: &mut PgConnection) -> Result<Vec<String>, Error> {
    instance::table
      .inner_join(federation_allowlist::table)
      .select(instance::domain)
      .load::<String>(conn)
  }

  pub fn blocklist(conn: &mut PgConnection) -> Result<Vec<String>, Error> {
    instance::table
      .inner_join(federation_blocklist::table)
      .select(instance::domain)
      .load::<String>(conn)
  }

  pub fn linked(conn: &mut PgConnection) -> Result<Vec<String>, Error> {
    instance::table
      .left_join(federation_blocklist::table)
      .filter(federation_blocklist::id.is_null())
      .select(instance::domain)
      .load::<String>(conn)
  }
}
