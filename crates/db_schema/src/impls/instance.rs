use crate::{
  newtypes::InstanceId,
  schema::{federation_allowlist, federation_blocklist, instance},
  source::instance::{Instance, InstanceForm},
  utils::{get_conn, naive_now, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

impl Instance {
  pub(crate) async fn read_or_create_with_conn(
    conn: &mut AsyncPgConnection,
    domain_: String,
  ) -> Result<Self, Error> {
    use crate::schema::instance::domain;
    // First try to read the instance row and return directly if found
    let instance = instance::table
      .filter(domain.eq(&domain_))
      .first::<Self>(conn)
      .await;
    match instance {
      Ok(i) => Ok(i),
      Err(diesel::NotFound) => {
        // Instance not in database yet, insert it
        let form = InstanceForm::builder()
          .domain(domain_)
          .updated(Some(naive_now()))
          .build();
        insert_into(instance::table)
          .values(&form)
          // Necessary because this method may be called concurrently for the same domain. This
          // could be handled with a transaction, but nested transactions arent allowed
          .on_conflict(instance::domain)
          .do_update()
          .set(&form)
          .get_result::<Self>(conn)
          .await
      }
      e => e,
    }
  }

  /// Attempt to read Instance column for the given domain. If it doesnt exist, insert a new one.
  /// There is no need for update as the domain of an existing instance cant change.
  pub async fn read_or_create(pool: &DbPool, domain: String) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    Self::read_or_create_with_conn(conn, domain).await
  }
  pub async fn delete(pool: &DbPool, instance_id: InstanceId) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(instance::table.find(instance_id))
      .execute(conn)
      .await
  }
  #[cfg(test)]
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
