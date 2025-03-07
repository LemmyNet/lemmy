use crate::{
  diesel::dsl::IntervalDsl,
  newtypes::{InstanceId, PersonId},
  schema::{
    federation_allowlist,
    federation_blocklist,
    federation_queue_state,
    instance,
    instance_actions,
    local_site,
    site,
  },
  source::{
    federation_queue_state::FederationQueueState,
    instance::{Instance, InstanceActions, InstanceBlockForm, InstanceForm},
  },
  traits::Blockable,
  utils::{
    functions::{coalesce, lower},
    get_conn,
    now,
    uplete,
    DbPool,
  },
};
use chrono::Utc;
use diesel::{
  dsl::{count_star, exists, insert_into, not, select},
  result::Error,
  ExpressionMethods,
  NullableExpressionMethods,
  OptionalExtension,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Instance {
  /// Attempt to read Instance column for the given domain. If it doesn't exist, insert a new one.
  /// There is no need for update as the domain of an existing instance cant change.
  pub async fn read_or_create(pool: &mut DbPool<'_>, domain_: String) -> Result<Self, Error> {
    use crate::schema::instance::domain;
    let conn = &mut get_conn(pool).await?;

    // First try to read the instance row and return directly if found
    let instance = instance::table
      .filter(lower(domain).eq(&domain_.to_lowercase()))
      .first(conn)
      .await
      .optional()?;

    // TODO could convert this to unwrap_or_else once async closures are stable
    match instance {
      Some(i) => Ok(i),
      None => {
        // Instance not in database yet, insert it
        let form = InstanceForm {
          updated: Some(Utc::now()),
          ..InstanceForm::new(domain_)
        };
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
    }
  }
  pub async fn read(pool: &mut DbPool<'_>, instance_id: InstanceId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    instance::table.find(instance_id).first(conn).await
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

  /// Only for use in tests
  pub async fn delete_all(pool: &mut DbPool<'_>) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(federation_queue_state::table)
      .execute(conn)
      .await?;
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

  /// returns a list of all instances, each with a flag of whether the instance is allowed or not
  /// and dead or not ordered by id
  pub async fn read_federated_with_blocked_and_dead(
    pool: &mut DbPool<'_>,
  ) -> Result<Vec<(Self, bool, bool)>, Error> {
    let conn = &mut get_conn(pool).await?;
    let is_dead_expr = coalesce(instance::updated, instance::published).lt(now() - 3.days());
    // this needs to be done in two steps because the meaning of the "blocked" column depends on the
    // existence of any value at all in the allowlist. (so a normal join wouldn't work)
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

impl Blockable for InstanceActions {
  type Form = InstanceBlockForm;
  type ObjectIdType = InstanceId;
  type ObjectType = Instance;

  async fn block(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(instance_actions::table)
      .values(form)
      .on_conflict((instance_actions::person_id, instance_actions::instance_id))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::InstanceBlockAlreadyExists)
  }

  async fn unblock(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(instance_actions::table.find((form.person_id, form.instance_id)))
      .set_null(instance_actions::blocked)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::InstanceBlockAlreadyExists)
  }

  async fn read_block(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    instance_id: Self::ObjectIdType,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    let find_action = instance_actions::table
      .find((person_id, instance_id))
      .filter(instance_actions::blocked.is_not_null());
    select(not(exists(find_action)))
      .get_result::<bool>(conn)
      .await?
      .then_some(())
      .ok_or(LemmyErrorType::InstanceIsBlocked.into())
  }

  async fn read_blocks_for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> Result<Vec<Self::ObjectType>, Error> {
    let conn = &mut get_conn(pool).await?;
    instance_actions::table
      .filter(instance_actions::blocked.is_not_null())
      .inner_join(instance::table)
      .select(instance::all_columns)
      .filter(instance_actions::person_id.eq(person_id))
      .order_by(instance_actions::blocked)
      .load::<Instance>(conn)
      .await
  }
}
