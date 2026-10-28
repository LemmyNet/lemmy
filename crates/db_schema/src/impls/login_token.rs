use crate::{
  diesel::{ExpressionMethods, QueryDsl},
  source::login_token::{LoginToken, LoginTokenCreateForm},
};
use diesel::{OptionalExtension, delete, dsl::exists, insert_into, select};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::{
  newtypes::LocalUserId,
  schema::login_token::{dsl::login_token, feed_only, user_id},
};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  sensitive::SensitiveString,
  traits::Crud,
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult, UntranslatedError};

impl LoginToken {
  pub async fn create(pool: &mut DbPool<'_>, form: LoginTokenCreateForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(login_token)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  /// Check if the given token is valid for user. `feed_only` must match the type of the
  /// stored token, so that feed-only tokens are rejected for normal api calls and vice versa.
  pub async fn validate(
    pool: &mut DbPool<'_>,
    user_id_: LocalUserId,
    token_: &str,
    feed_only_: bool,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      login_token
        .find(token_)
        .filter(user_id.eq(user_id_))
        .filter(feed_only.eq(feed_only_)),
    ))
    .get_result::<bool>(conn)
    .await?
    .then_some(())
    .ok_or(LemmyErrorType::NotLoggedIn.into())
  }

  pub async fn list(pool: &mut DbPool<'_>, user_id_: LocalUserId) -> LemmyResult<Vec<LoginToken>> {
    let conn = &mut get_conn(pool).await?;

    login_token
      .filter(user_id.eq(user_id_))
      .filter(feed_only.eq(false))
      .get_results(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Returns existing feed-only token for the user, if any.
  pub async fn read_feed_only(
    pool: &mut DbPool<'_>,
    user_id_: LocalUserId,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    login_token
      .filter(user_id.eq(user_id_))
      .filter(feed_only.eq(true))
      .first::<Self>(conn)
      .await
      .optional()
      .map_err(Into::into)
  }

  /// Invalidate specific token on user logout.
  pub async fn invalidate(pool: &mut DbPool<'_>, token_: &str) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    delete(login_token.find(token_))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::Deleted)
  }
  /// Invalidate all logins of given user on password reset/change, or account deletion.
  /// Feed-only tokens are kept so that existing RSS feed urls keep working.
  pub async fn invalidate_all(pool: &mut DbPool<'_>, user_id_: LocalUserId) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    delete(
      login_token
        .filter(user_id.eq(user_id_))
        .filter(feed_only.eq(false)),
    )
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::Deleted)
  }
}

impl Crud for LoginToken {
  type InsertForm = LoginTokenCreateForm;
  type UpdateForm = LoginTokenCreateForm;
  type IdType = SensitiveString;

  /// Use [[LoginToken::create]]
  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    LoginToken::create(pool, form.clone()).await
  }

  /// Login tokens cannot be updated.
  async fn update(
    _pool: &mut DbPool<'_>,
    _token: Self::IdType,
    _form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    Err(UntranslatedError::Unreachable.into())
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use crate::source::{
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm},
    login_token::{LoginToken, LoginTokenCreateForm},
    person::{Person, PersonInsertForm},
  };
  use lemmy_diesel_utils::{
    connection::{DbPool, build_db_pool_for_tests},
    sensitive::SensitiveString,
    traits::Crud,
  };
  use lemmy_utils::error::{LemmyErrorType, LemmyResult};
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  async fn setup(pool: &mut DbPool<'_>) -> LemmyResult<(Instance, Person, LocalUser)> {
    let instance = Instance::read_or_create(pool, "other_domain.tld").await?;
    let person =
      Person::create(pool, &PersonInsertForm::test_form(instance.id, "feed_user")).await?;
    let local_user =
      LocalUser::create(pool, &LocalUserInsertForm::test_form(person.id), vec![]).await?;
    Ok((instance, person, local_user))
  }

  async fn normal_create_form(
    pool: &mut DbPool<'_>,
    user_id: lemmy_db_schema_file::newtypes::LocalUserId,
  ) -> LemmyResult<LoginToken> {
    LoginToken::create(
      pool,
      LoginTokenCreateForm {
        token: "normal_jwt".to_owned().into(),
        user_id,
        ip: None,
        user_agent: None,
        feed_only: false,
      },
    )
    .await
  }

  async fn get_or_create_feed_only(
    pool: &mut DbPool<'_>,
    user_id: lemmy_db_schema_file::newtypes::LocalUserId,
    token: SensitiveString,
  ) -> LemmyResult<LoginToken> {
    if let Some(existing) = LoginToken::read_feed_only(pool, user_id).await? {
      return Ok(existing);
    }
    LoginToken::create(
      pool,
      LoginTokenCreateForm {
        token,
        user_id,
        ip: None,
        user_agent: None,
        feed_only: true,
      },
    )
    .await
  }

  #[tokio::test]
  #[serial]
  async fn feed_only_token_validate_and_persist() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let (instance, person, local_user) = setup(pool).await?;

    let token: SensitiveString = "feed_jwt".to_owned().into();
    let first = LoginToken::get_or_create_feed_only(pool, local_user.id, token).await?;
    assert!(first.feed_only);

    // calling again returns the same token, no regeneration
    let second =
      LoginToken::get_or_create_feed_only(pool, local_user.id, "other_jwt".to_owned().into())
        .await?;
    assert_eq!(first.token, second.token);

    // feed token validates with feed_only=true
    LoginToken::validate(pool, local_user.id, &first.token, true).await?;
    let res = LoginToken::validate(pool, local_user.id, &first.token, false).await;
    assert_eq!(
      res.err().map(|e| e.error_type),
      Some(LemmyErrorType::NotLoggedIn)
    );

    // normal token rejected with feed_only=true
    normal_create_form(pool, local_user.id).await?;
    LoginToken::validate(pool, local_user.id, "normal_jwt", false).await?;
    let res = LoginToken::validate(pool, local_user.id, "normal_jwt", true).await;
    assert_eq!(
      res.err().map(|e| e.error_type),
      Some(LemmyErrorType::NotLoggedIn)
    );

    Person::delete(pool, person.id).await?;
    Instance::delete(pool, instance.id).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn list_excludes_feed_only() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let (instance, person, local_user) = setup(pool).await?;

    normal_create_form(pool, local_user.id).await?;
    LoginToken::get_or_create_feed_only(pool, local_user.id, "feed_jwt_list".to_owned().into())
      .await?;

    let logins = LoginToken::list(pool, local_user.id).await?;
    assert_eq!(1, logins.len());
    let expected: SensitiveString = "normal_jwt".to_owned().into();
    assert_eq!(expected, logins[0].token);

    Person::delete(pool, person.id).await?;
    Instance::delete(pool, instance.id).await?;
    Ok(())
  }
}
