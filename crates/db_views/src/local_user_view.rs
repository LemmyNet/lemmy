use crate::structs::LocalUserView;
use diesel::{result::Error, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aggregates::structs::PersonAggregates,
  newtypes::{LocalUserId, PersonId},
  schema::{local_user, person, person_aggregates},
  source::{local_user::LocalUser, person::Person},
  utils::{functions::lower, DbConn, DbPool, ListFn, Queries, ReadFn},
};

enum ReadBy<'a> {
  Id(LocalUserId),
  Person(PersonId),
  Name(&'a str),
  NameOrEmail(&'a str),
  Email(&'a str),
}

enum ListMode {
  AdminsWithEmails,
}

fn queries<'a>(
) -> Queries<impl ReadFn<'a, LocalUserView, ReadBy<'a>>, impl ListFn<'a, LocalUserView, ListMode>> {
  let selection = (
    local_user::all_columns,
    person::all_columns,
    person_aggregates::all_columns,
  );

  let read = move |mut conn: DbConn<'a>, search: ReadBy<'a>| async move {
    let mut query = local_user::table.into_boxed();
    query = match search {
      ReadBy::Id(local_user_id) => query.filter(local_user::id.eq(local_user_id)),
      ReadBy::Email(from_email) => query.filter(local_user::email.eq(from_email)),
      _ => query,
    };
    let mut query = query.inner_join(person::table);
    query = match search {
      ReadBy::Person(person_id) => query.filter(person::id.eq(person_id)),
      ReadBy::Name(name) => query.filter(lower(person::name).eq(name.to_lowercase())),
      ReadBy::NameOrEmail(name_or_email) => query.filter(
        lower(person::name)
          .eq(lower(name_or_email))
          .or(local_user::email.eq(name_or_email)),
      ),
      _ => query,
    };
    query
      .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
      .select(selection)
      .first::<LocalUserView>(&mut conn)
      .await
  };

  let list = move |mut conn: DbConn<'a>, mode: ListMode| async move {
    match mode {
      ListMode::AdminsWithEmails => {
        local_user::table
          .filter(local_user::email.is_not_null())
          .filter(local_user::admin.eq(true))
          .inner_join(person::table)
          .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
          .select(selection)
          .load::<LocalUserView>(&mut conn)
          .await
      }
    }
  };

  Queries::new(read, list)
}

impl LocalUserView {
  pub async fn read(pool: &mut DbPool<'_>, local_user_id: LocalUserId) -> Result<Self, Error> {
    queries().read(pool, ReadBy::Id(local_user_id)).await
  }

  pub async fn read_person(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Self, Error> {
    queries().read(pool, ReadBy::Person(person_id)).await
  }

  pub async fn read_from_name(pool: &mut DbPool<'_>, name: &str) -> Result<Self, Error> {
    queries().read(pool, ReadBy::Name(name)).await
  }

  pub async fn find_by_email_or_name(
    pool: &mut DbPool<'_>,
    name_or_email: &str,
  ) -> Result<Self, Error> {
    queries()
      .read(pool, ReadBy::NameOrEmail(name_or_email))
      .await
  }

  pub async fn find_by_email(pool: &mut DbPool<'_>, from_email: &str) -> Result<Self, Error> {
    queries().read(pool, ReadBy::Email(from_email)).await
  }

  pub async fn list_admins_with_emails(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    queries().list(pool, ListMode::AdminsWithEmails).await
  }
}
