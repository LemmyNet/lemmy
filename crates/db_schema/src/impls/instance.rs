use crate::{
  newtypes::InstanceId,
  schema::{federation_allowlist, federation_blocklist, instance},
  source::instance::{Instance, InstanceForm},
  utils::{get_conn, naive_now, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_utils::utils::generate_domain_url;
use url::Url;

impl Instance {
  async fn create_from_form_conn(
    conn: &mut AsyncPgConnection,
    form: &InstanceForm,
  ) -> Result<Self, Error> {
    // Do upsert on domain name conflict
    insert_into(instance::table)
      .values(form)
      .on_conflict(instance::domain)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn create(pool: &DbPool, domain: &str) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    Self::create_conn(conn, domain).await
  }
  pub async fn create_from_actor_id(pool: &DbPool, actor_id: &Url) -> Result<Self, Error> {
    let domain = &generate_domain_url(actor_id).expect("actor id missing a domain");
    Self::create(pool, domain).await
  }
  pub async fn create_conn(conn: &mut AsyncPgConnection, domain: &str) -> Result<Self, Error> {
    let form = InstanceForm {
      domain: domain.to_string(),
      updated: Some(naive_now()),
    };
    Self::create_from_form_conn(conn, &form).await
  }
  pub async fn delete(pool: &DbPool, instance_id: InstanceId) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(instance::table.find(instance_id))
      .execute(conn)
      .await
  }
  pub async fn delete_all(pool: &DbPool) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(instance::table).execute(conn).await
  }
  pub async fn allowlist(pool: &DbPool) -> Result<Vec<String>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance::table
      .inner_join(federation_allowlist::table)
      .select(instance::domain)
      .load::<String>(conn)
      .await
  }

  pub async fn blocklist(pool: &DbPool) -> Result<Vec<String>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance::table
      .inner_join(federation_blocklist::table)
      .select(instance::domain)
      .load::<String>(conn)
      .await
  }

  pub async fn linked(pool: &DbPool) -> Result<Vec<String>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance::table
      .left_join(federation_blocklist::table)
      .filter(federation_blocklist::id.is_null())
      .select(instance::domain)
      .load::<String>(conn)
      .await
  }
}
