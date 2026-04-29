use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use i_love_jesus::SortDirection;
use lemmy_db_schema::{
  newtypes::LocalUserId,
  source::local_user_invite::{LocalUserInvite, invitation_keys as key},
  utils::limit_fetch,
};
use lemmy_db_schema_file::schema::local_user_invite;
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  pagination::{PagedResponse, PaginationCursor, PaginationCursorConversion, paginate_response},
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[derive(Default)]
pub struct LocalUserInviteQuery {
  pub local_user_id: LocalUserId,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}

impl LocalUserInviteQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> LemmyResult<PagedResponse<LocalUserInvite>> {
    let limit = limit_fetch(self.limit, None)?;

    let mut query = local_user_invite::table
      .select(LocalUserInvite::as_select())
      .limit(limit)
      .into_boxed();

    query = query.filter(local_user_invite::local_user_id.eq(self.local_user_id));

    let paginated_query =
      LocalUserInvite::paginate(query, &self.page_cursor, SortDirection::Asc, pool)
        .await?
        .then_order_by(key::published_at)
        .then_order_by(key::id);

    let conn = &mut get_conn(pool).await?;
    let res = paginated_query
      .load::<LocalUserInvite>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)?;
    paginate_response(res, limit, self.page_cursor)
  }

  pub async fn count(self, pool: &mut DbPool<'_>) -> LemmyResult<i64> {
    use diesel::dsl::count_star;

    let conn = &mut get_conn(pool).await?;
    let mut query = local_user_invite::table.select(count_star()).into_boxed();

    query = query.filter(local_user_invite::local_user_id.eq(self.local_user_id));

    query
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

#[cfg(test)]
mod tests {
  use crate::impls::LocalUserInviteQuery;
  use lemmy_db_schema::{
    newtypes::LocalUserId,
    source::{
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      local_user_invite::{LocalUserInvite, LocalUserInviteInsertForm},
      person::{Person, PersonInsertForm},
    },
  };
  use lemmy_diesel_utils::{
    connection::{DbPool, build_db_pool_for_tests},
    traits::Crud,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  async fn create_local_user(
    pool: &mut DbPool<'_>,
    instance: &Instance,
    name: &str,
  ) -> LemmyResult<(Person, LocalUser)> {
    let person = Person::create(pool, &PersonInsertForm::test_form(instance.id, name)).await?;
    let local_user =
      LocalUser::create(pool, &LocalUserInsertForm::test_form(person.id), vec![]).await?;
    Ok((person, local_user))
  }

  async fn create_invite(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
    token: &str,
  ) -> LemmyResult<LocalUserInvite> {
    LocalUserInvite::create(
      pool,
      &LocalUserInviteInsertForm {
        token: token.to_string(),
        local_user_id,
        max_uses: None,
        expires_at: None,
      },
    )
    .await
  }

  async fn list(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
  ) -> LemmyResult<Vec<LocalUserInvite>> {
    Ok(
      LocalUserInviteQuery {
        local_user_id,
        ..Default::default()
      }
      .list(pool)
      .await?
      .items,
    )
  }

  async fn count(pool: &mut DbPool<'_>, local_user_id: LocalUserId) -> LemmyResult<i64> {
    LocalUserInviteQuery {
      local_user_id,
      ..Default::default()
    }
    .count(pool)
    .await
  }

  #[tokio::test]
  #[serial]
  async fn test_list_and_count() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let instance = Instance::read_or_create(pool, "my_domain.tld").await?;
    let (timmy_person, timmy_local_user) = create_local_user(pool, &instance, "timmy_inv").await?;

    let invite_1 = create_invite(pool, timmy_local_user.id, "token_one").await?;
    let invite_2 = create_invite(pool, timmy_local_user.id, "token_two").await?;

    assert_eq!(list(pool, timmy_local_user.id).await?, [invite_1, invite_2]);
    assert_eq!(count(pool, timmy_local_user.id).await?, 2);

    Person::delete(pool, timmy_person.id).await?;
    Instance::delete(pool, instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_list_filters_by_user() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let instance = Instance::read_or_create(pool, "my_domain.tld").await?;
    let (timmy_person, timmy_local_user) = create_local_user(pool, &instance, "timmy_inv2").await?;
    let (sara_person, sara_local_user) = create_local_user(pool, &instance, "sara_inv").await?;

    let timmy_invite = create_invite(pool, timmy_local_user.id, "timmy_token").await?;
    let sara_invite = create_invite(pool, sara_local_user.id, "sara_token").await?;

    assert_eq!(list(pool, timmy_local_user.id).await?, [timmy_invite]);
    assert_eq!(list(pool, sara_local_user.id).await?, [sara_invite]);
    assert_eq!(count(pool, timmy_local_user.id).await?, 1);
    assert_eq!(count(pool, sara_local_user.id).await?, 1);

    Person::delete(pool, timmy_person.id).await?;
    Person::delete(pool, sara_person.id).await?;
    Instance::delete(pool, instance.id).await?;

    Ok(())
  }
}
