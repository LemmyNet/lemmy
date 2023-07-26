use crate::structs::LocalUserView;
use actix_web::{dev::Payload, FromRequest, HttpMessage, HttpRequest};
use diesel::{result::Error, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aggregates::structs::PersonAggregates,
  newtypes::{LocalUserId, PersonId},
  schema::{local_user, person, person_aggregates},
  source::{local_user::LocalUser, person::Person},
  traits::JoinView,
  utils::{functions::lower, get_conn, DbPool},
};
use lemmy_utils::error::{LemmyError, LemmyErrorType};
use std::future::{ready, Ready};

type LocalUserViewTuple = (LocalUser, Person, PersonAggregates);

impl LocalUserView {
  pub async fn read(pool: &mut DbPool<'_>, local_user_id: LocalUserId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    let (local_user, person, counts) = local_user::table
      .find(local_user_id)
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)
      .await?;
    Ok(Self {
      local_user,
      person,
      counts,
    })
  }

  pub async fn read_person(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let (local_user, person, counts) = local_user::table
      .filter(person::id.eq(person_id))
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)
      .await?;
    Ok(Self {
      local_user,
      person,
      counts,
    })
  }

  pub async fn read_from_name(pool: &mut DbPool<'_>, name: &str) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let (local_user, person, counts) = local_user::table
      .filter(lower(person::name).eq(name.to_lowercase()))
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)
      .await?;
    Ok(Self {
      local_user,
      person,
      counts,
    })
  }

  pub async fn find_by_email_or_name(
    pool: &mut DbPool<'_>,
    name_or_email: &str,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let (local_user, person, counts) = local_user::table
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .filter(
        lower(person::name)
          .eq(lower(name_or_email))
          .or(local_user::email.eq(name_or_email)),
      )
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)
      .await?;
    Ok(Self {
      local_user,
      person,
      counts,
    })
  }

  pub async fn find_by_email(pool: &mut DbPool<'_>, from_email: &str) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let (local_user, person, counts) = local_user::table
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .filter(local_user::email.eq(from_email))
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
      ))
      .first::<LocalUserViewTuple>(conn)
      .await?;
    Ok(Self {
      local_user,
      person,
      counts,
    })
  }

  pub async fn list_admins_with_emails(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = local_user::table
      .filter(person::admin.eq(true))
      .filter(local_user::email.is_not_null())
      .inner_join(person::table)
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .select((
        local_user::all_columns,
        person::all_columns,
        person_aggregates::all_columns,
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
    }
  }
}

impl FromRequest for LocalUserView {
  type Error = LemmyError;
  type Future = Ready<Result<Self, Self::Error>>;

  fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
    ready(match req.extensions().get::<LocalUserView>() {
      Some(c) => Ok(c.clone()),
      None => Err(LemmyErrorType::IncorrectLogin.into()),
    })
  }
}
