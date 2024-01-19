use crate::{
  diesel::dsl::IntervalDsl,
  newtypes::InstanceId,
  schema::{
    federation_allowlist,
    federation_blocklist,
    federation_queue_state,
    instance,
    local_site,
    site,
  },
  source::{
    federation_queue_state::FederationQueueState,
    instance::{Instance, InstanceForm},
  },
  utils::{
    functions::{coalesce, lower},
    get_conn,
    naive_now,
    now,
    DbPool,
  },
};
use diesel::{
  dsl::{count_star, insert_into},
  result::Error,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
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
  pub async fn update(
    pool: &mut DbPool<'_>,
    instance_id: InstanceId,
    form: InstanceForm,
  ) -> Result<usize, Error> {
    let mut conn = get_conn(pool).await?;
    diesel::update(instance::table.find(instance_id))
      .set(form)
      .execute(&mut conn)
      .await
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
      .select(Self::as_select())
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
      .select(Self::as_select())
      .get_results(conn)
      .await
  }

  pub async fn blocklist(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance::table
      .inner_join(federation_blocklist::table)
      .select(Self::as_select())
      .get_results(conn)
      .await
  }

  /// returns a list of all instances, each with a flag of whether the instance is allowed or not and dead or not
  /// ordered by id
  pub async fn read_federated_with_blocked_and_dead(
    pool: &mut DbPool<'_>,
  ) -> Result<Vec<(Self, bool, bool)>, Error> {
    let conn = &mut get_conn(pool).await?;
    let is_dead_expr = coalesce(instance::updated, instance::published).lt(now() - 3.days());
    // this needs to be done in two steps because the meaning of the "blocked" column depends on the existence
    // of any value at all in the allowlist. (so a normal join wouldn't work)
    let use_allowlist = federation_allowlist::table
      .select(count_star().gt(0))
      .get_result::<bool>(conn)
      .await?;
    if use_allowlist {
      instance::table
        .left_join(federation_allowlist::table)
        .select((
          Self::as_select(),
          federation_allowlist::instance_id.nullable().is_not_null(),
          is_dead_expr,
        ))
        .order_by(instance::id)
        .get_results::<(Self, bool, bool)>(conn)
        .await
    } else {
      instance::table
        .left_join(federation_blocklist::table)
        .select((
          Self::as_select(),
          federation_blocklist::instance_id.nullable().is_null(),
          is_dead_expr,
        ))
        .order_by(instance::id)
        .get_results::<(Self, bool, bool)>(conn)
        .await
    }
  }

  /// returns (instance, blocked, allowed, fed queue state) tuples
  pub async fn read_all_with_fed_state(
    pool: &mut DbPool<'_>,
  ) -> Result<Vec<(Self, Option<FederationQueueState>, bool, bool)>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance::table
      // omit instance representing the local site
      .left_join(site::table.inner_join(local_site::table))
      .filter(local_site::id.is_null())
      .left_join(federation_blocklist::table)
      .left_join(federation_allowlist::table)
      .left_join(federation_queue_state::table)
      .select((
        Self::as_select(),
        Option::<FederationQueueState>::as_select(),
        federation_blocklist::instance_id.nullable().is_not_null(),
        federation_allowlist::instance_id.nullable().is_not_null(),
      ))
      .get_results(conn)
      .await
  }
}
