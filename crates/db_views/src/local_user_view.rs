use crate::structs::LocalUserView;
use diesel::{result::Error, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aggregates::structs::PersonAggregates,
  newtypes::{LocalUserId, PersonId},
  schema::{local_site, local_user, person, person_aggregates, site_role},
  source::{local_user::LocalUser, person::Person, site_role::SiteRole},
  traits::JoinView,
  utils::{functions::lower, get_conn, DbPool},
};

type LocalUserViewTuple = (LocalUser, Person, PersonAggregates, SiteRole);

impl LocalUserView {
  pub async fn read(pool: &DbPool, local_user_id: LocalUserId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    let (local_user, person, counts, site_role) = local_user::table
      .find(local_user_id)
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .inner_join(site_role::table.on(site_role::id.eq(person::site_role_id)))
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
        site_role::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)
      .await?;
    Ok(Self {
      local_user,
      person,
      counts,
      site_role,
    })
  }

  pub async fn read_person(pool: &DbPool, person_id: PersonId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let (local_user, person, counts, site_role) = local_user::table
      .filter(person::id.eq(person_id))
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .inner_join(site_role::table.on(site_role::id.eq(person::site_role_id)))
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
        site_role::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)
      .await?;
    Ok(Self {
      local_user,
      person,
      counts,
      site_role,
    })
  }

  pub async fn read_from_name(pool: &DbPool, name: &str) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let (local_user, person, counts, site_role) = local_user::table
      .filter(lower(person::name).eq(name.to_lowercase()))
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .inner_join(site_role::table.on(site_role::id.eq(person::site_role_id)))
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
        site_role::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)
      .await?;
    Ok(Self {
      local_user,
      person,
      counts,
      site_role,
    })
  }

  pub async fn find_by_email_or_name(pool: &DbPool, name_or_email: &str) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let (local_user, person, counts, site_role) = local_user::table
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .inner_join(site_role::table.on(site_role::id.eq(person::site_role_id)))
      .filter(
        lower(person::name)
          .eq(lower(name_or_email))
          .or(local_user::email.eq(name_or_email)),
      )
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
        site_role::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)
      .await?;
    Ok(Self {
      local_user,
      person,
      counts,
      site_role,
    })
  }

  pub async fn find_by_email(pool: &DbPool, from_email: &str) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let (local_user, person, counts, site_role) = local_user::table
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .inner_join(site_role::table.on(site_role::id.eq(person::site_role_id)))
      .filter(local_user::email.eq(from_email))
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
        site_role::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)
      .await?;
    Ok(Self {
      local_user,
      person,
      counts,
      site_role,
    })
  }

  pub async fn list_admins_with_emails(pool: &DbPool) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = local_user::table
      .inner_join(person::table)
      .inner_join(site_role::table.on(site_role::id.eq(person::site_role_id)))
      .inner_join(local_site::table.on(local_site::top_admin_role_id.eq(site_role::id)))
      .filter(local_user::email.is_not_null())
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
        site_role::all_columns,
      ))
      .load::<LocalUserViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(LocalUserView::from_tuple).collect())
  }
}

impl JoinView for LocalUserView {
  type JoinTuple = LocalUserViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      local_user: a.0,
      person: a.1,
      counts: a.2,
      site_role: a.3,
    }
  }
}
