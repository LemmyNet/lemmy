use crate::structs::LocalUserView;
use actix_web::{dev::Payload, FromRequest, HttpMessage, HttpRequest};
use diesel::{result::Error, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{LocalUserId, OAuthProviderId, PersonId},
  schema::{local_user, local_user_vote_display_mode, oauth_account, person, person_aggregates},
  source::{
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
  },
  traits::Crud,
  utils::{
    functions::{coalesce, lower},
    get_conn,
    DbPool,
  },
};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult};
use std::future::{ready, Ready};

#[diesel::dsl::auto_type]
fn joins() -> _ {
  local_user::table
    .inner_join(local_user_vote_display_mode::table)
    .inner_join(person::table)
    .inner_join(person_aggregates::table.on(person::id.eq(person_aggregates::person_id)))
}

type SelectionType = (
  <local_user::table as diesel::Table>::AllColumns,
  <local_user_vote_display_mode::table as diesel::Table>::AllColumns,
  <person::table as diesel::Table>::AllColumns,
  <person_aggregates::table as diesel::Table>::AllColumns,
);

const SELECTION: SelectionType = (
  local_user::all_columns,
  local_user_vote_display_mode::all_columns,
  person::all_columns,
  person_aggregates::all_columns,
);

impl LocalUserView {
  pub async fn read(pool: &mut DbPool<'_>, local_user_id: LocalUserId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    joins()
      .filter(local_user::id.eq(local_user_id))
      .select(SELECTION)
      .first(conn)
      .await
  }

  pub async fn read_person(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    joins()
      .filter(person::id.eq(person_id))
      .select(SELECTION)
      .first(conn)
      .await
  }

  pub async fn read_from_name(pool: &mut DbPool<'_>, name: &str) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    joins()
      .filter(lower(person::name).eq(name.to_lowercase()))
      .select(SELECTION)
      .first(conn)
      .await
  }

  pub async fn find_by_email_or_name(
    pool: &mut DbPool<'_>,
    name_or_email: &str,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    joins()
      .filter(
        lower(person::name)
          .eq(lower(name_or_email.to_lowercase()))
          .or(lower(coalesce(local_user::email, "")).eq(name_or_email.to_lowercase())),
      )
      .select(SELECTION)
      .first(conn)
      .await
  }

  pub async fn find_by_email(pool: &mut DbPool<'_>, from_email: &str) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    joins()
      .filter(lower(coalesce(local_user::email, "")).eq(from_email.to_lowercase()))
      .select(SELECTION)
      .first(conn)
      .await
  }

  pub async fn find_by_oauth_id(
    pool: &mut DbPool<'_>,
    oauth_provider_id: OAuthProviderId,
    oauth_user_id: &str,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    joins()
      .inner_join(oauth_account::table)
      .filter(oauth_account::oauth_provider_id.eq(oauth_provider_id))
      .filter(oauth_account::oauth_user_id.eq(oauth_user_id))
      .select(SELECTION)
      .first(conn)
      .await
  }

  pub async fn list_admins_with_emails(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    joins()
      .filter(local_user::email.is_not_null())
      .filter(local_user::admin.eq(true))
      .select(SELECTION)
      .load::<Self>(conn)
      .await
  }

  pub async fn create_test_user(
    pool: &mut DbPool<'_>,
    name: &str,
    bio: &str,
    admin: bool,
  ) -> LemmyResult<Self> {
    let instance_id = Instance::read_or_create(pool, "example.com".to_string())
      .await?
      .id;
    let person_form = PersonInsertForm {
      display_name: Some(name.to_owned()),
      bio: Some(bio.to_owned()),
      ..PersonInsertForm::test_form(instance_id, name)
    };
    let person = Person::create(pool, &person_form).await?;

    let user_form = match admin {
      true => LocalUserInsertForm::test_form_admin(person.id),
      false => LocalUserInsertForm::test_form(person.id),
    };
    let local_user = LocalUser::create(pool, &user_form, vec![]).await?;

    LocalUserView::read(pool, local_user.id)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
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