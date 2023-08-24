use crate::{
  diesel::dsl::IntervalDsl,
  newtypes::InstanceId,
  schema::{federation_allowlist, federation_blocklist, instance, local_site, site},
  source::instance::{Instance, InstanceForm},
  utils::{functions::lower, get_conn, naive_now, now, DbPool},
};
use diesel::{
  dsl::insert_into,
  result::Error,
  sql_types::{Nullable, Timestamptz},
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl Instance {
  /// Attempt to read Instance column for the given domain. If it doesnt exist, insert a new one.
  /// There is no need for update as the domain of an existing instance cant change.
  pub async fn read_or_create(pool: &mut DbPool<'_>, domain_: String) -> Result<Self, Error> {
    use crate::schema::instance::domain;
    let conn = &mut get_conn(pool).await?;

    // First try to read the instance row and return directly if found
    let instance = instance::table
      .filter(lower(domain).eq(&domain_.to_lowercase()))
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
  pub async fn delete(pool: &mut DbPool<'_>, instance_id: InstanceId) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(instance::table.find(instance_id))
      .execute(conn)
      .await
  }

  pub async fn read_all(pool: &mut DbPool<'_>) -> Result<Vec<Instance>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance::table
      .select(instance::all_columns)
      .get_results(conn)
      .await
  }

  pub async fn dead_instances(pool: &mut DbPool<'_>) -> Result<Vec<String>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance::table
      .select(instance::domain)
      .filter(coalesce(instance::updated, instance::published).lt(now() - 3.days()))
      .get_results(conn)
      .await
  }

  #[cfg(test)]
  pub async fn delete_all(pool: &mut DbPool<'_>) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(instance::table).execute(conn).await
  }
  pub async fn allowlist(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance::table
      .inner_join(federation_allowlist::table)
      .select(instance::all_columns)
      .get_results(conn)
      .await
  }

  pub async fn blocklist(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance::table
      .inner_join(federation_blocklist::table)
      .select(instance::all_columns)
      .get_results(conn)
      .await
  }

  pub async fn linked(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance::table
      // omit instance representing the local site
      .left_join(site::table.inner_join(local_site::table))
      .filter(local_site::id.is_null())
      // omit instances in the blocklist
      .left_join(federation_blocklist::table)
      .filter(federation_blocklist::id.is_null())
      .select(instance::all_columns)
      .get_results(conn)
      .await
  }
}

sql_function! { fn coalesce(x: Nullable<Timestamptz>, y: Timestamptz) -> Timestamptz; }
