use crate::LocalUserView;
use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload};
use diesel::{
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use i_love_jesus::asc_if;
use lemmy_db_schema::{
  LocalUserSortType,
  newtypes::{LocalUserId, OAuthProviderId},
  source::{
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm, person_keys},
  },
};
use lemmy_db_schema_file::{
  PersonId,
  aliases::creator_home_instance_actions,
  joins::creator_home_instance_actions_join,
  schema::{instance_actions, local_user, oauth_account, person},
};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  pagination::{
    CursorData,
    PagedResponse,
    PaginationCursor,
    PaginationCursorConversion,
    paginate_response,
  },
  traits::Crud,
  utils::{
    functions::{coalesce, lower},
    now,
  },
};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult};
use std::future::{Ready, ready};

impl LocalUserView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    local_user::table
      .inner_join(person::table)
      .left_join(creator_home_instance_actions_join())
  }

  pub async fn read(pool: &mut DbPool<'_>, local_user_id: LocalUserId) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(local_user::id.eq(local_user_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn read_person(pool: &mut DbPool<'_>, person_id: PersonId) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(person::id.eq(person_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn read_from_name(pool: &mut DbPool<'_>, name: &str) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(lower(person::name).eq(name.to_lowercase()))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn find_by_email_or_name(
    pool: &mut DbPool<'_>,
    name_or_email: &str,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(
        lower(person::name)
          .eq(lower(name_or_email.to_lowercase()))
          .or(lower(coalesce(local_user::email, "")).eq(name_or_email.to_lowercase())),
      )
      .select(Self::as_select())
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn find_by_email(pool: &mut DbPool<'_>, from_email: &str) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(lower(coalesce(local_user::email, "")).eq(from_email.to_lowercase()))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn find_by_oauth_id(
    pool: &mut DbPool<'_>,
    oauth_provider_id: OAuthProviderId,
    oauth_user_id: &str,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .inner_join(oauth_account::table)
      .filter(oauth_account::oauth_provider_id.eq(oauth_provider_id))
      .filter(oauth_account::oauth_user_id.eq(oauth_user_id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn list_admins_with_emails(pool: &mut DbPool<'_>) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    Self::joins()
      .filter(local_user::email.is_not_null())
      .filter(local_user::admin.eq(true))
      .select(Self::as_select())
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn create_test_user(
    pool: &mut DbPool<'_>,
    name: &str,
    bio: &str,
    admin: bool,
  ) -> LemmyResult<Self> {
    let instance_id = Instance::read_or_create(pool, "example.com").await?.id;
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

    LocalUserView::read(pool, local_user.id).await
  }
}

#[derive(Default)]
pub struct LocalUserQuery {
  pub banned_only: Option<bool>,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
  pub sort: Option<LocalUserSortType>,
}

impl LocalUserQuery {
  // TODO: add filters and sorts
  pub async fn list(self, pool: &mut DbPool<'_>) -> LemmyResult<PagedResponse<LocalUserView>> {
    let limit = self.limit.unwrap_or(i64::MAX);
    let mut query = LocalUserView::joins()
      .filter(person::deleted.eq(false))
      .limit(limit)
      .select(LocalUserView::as_select())
      .into_boxed();

    if self.banned_only.unwrap_or_default() {
      let actions = creator_home_instance_actions;

      query = query.filter(
        actions
          .field(instance_actions::received_ban_at)
          .is_not_null()
          .and(
            actions
              .field(instance_actions::ban_expires_at)
              .is_null()
              .or(
                actions
                  .field(instance_actions::ban_expires_at)
                  .gt(now().nullable()),
              ),
          ),
      );
    }

    // Only sort by ascending for Old
    let sort = self.sort.unwrap_or_default();
    let sort_direction = asc_if(sort == LocalUserSortType::Old);

    let paginated_query =
      LocalUserView::paginate(query, &self.page_cursor, sort_direction, pool, None)
        .await?
        .then_order_by(person_keys::published_at)
        // Tie breaker
        .then_order_by(person_keys::id);

    let conn = &mut get_conn(pool).await?;
    let res = paginated_query.load::<LocalUserView>(conn).await?;
    paginate_response(res, limit, self.page_cursor)
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

impl PaginationCursorConversion for LocalUserView {
  type PaginatedType = Person;

  fn to_cursor(&self) -> CursorData {
    CursorData::new_id(self.person.id.0)
  }

  async fn from_cursor(
    cursor: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    Person::read(pool, PersonId(cursor.id()?)).await
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use super::*;
  use lemmy_db_schema::{
    assert_length,
    source::{
      instance::{Instance, InstanceActions, InstanceBanForm},
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
    },
    traits::Bannable,
  };
  use lemmy_diesel_utils::{
    connection::{DbPool, build_db_pool_for_tests},
    traits::Crud,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  struct Data {
    alice: Person,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld").await?;

    let alice_form = PersonInsertForm {
      local: Some(true),
      ..PersonInsertForm::test_form(instance.id, "alice")
    };
    let alice = Person::create(pool, &alice_form).await?;
    let alice_local_user_form = LocalUserInsertForm::test_form(alice.id);
    LocalUser::create(pool, &alice_local_user_form, vec![]).await?;

    Ok(Data { alice })
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    Instance::delete(pool, data.alice.instance_id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn list_banned() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    InstanceActions::ban(
      pool,
      &InstanceBanForm::new(data.alice.id, data.alice.instance_id, None),
    )
    .await?;

    let list = LocalUserQuery {
      banned_only: Some(true),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_length!(1, list);
    assert_eq!(list[0].person.id, data.alice.id);

    cleanup(data, pool).await
  }
}
