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

impl Crud for LoginToken {
  type InsertForm = LoginTokenCreateForm;
  type UpdateForm = LoginTokenCreateForm;
  type IdType = SensitiveString;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(login_token)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    _pool: &mut DbPool<'_>,
    _token: Self::IdType,
    _form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    Err(UntranslatedError::Unreachable.into())
  }
}

impl LoginToken {
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
  pub async fn invalidate_all(pool: &mut DbPool<'_>, user_id_: LocalUserId) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    delete(login_token.filter(user_id.eq(user_id_)))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::Deleted)
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
    connection::build_db_pool_for_tests,
    sensitive::SensitiveString,
    traits::Crud,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn feed_only_token() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let instance = Instance::read_or_create(pool, "other_domain.tld").await?;
    let person =
      Person::create(pool, &PersonInsertForm::test_form(instance.id, "feed_user")).await?;
    let local_user =
      LocalUser::create(pool, &LocalUserInsertForm::test_form(person.id), vec![]).await?;

    let feed_token = SensitiveString::from("feed_jwt".to_string());
    let form = LoginTokenCreateForm {
      token: feed_token.clone(),
      user_id: local_user.id,
      ip: None,
      user_agent: None,
      feed_only: true,
    };
    LoginToken::create(pool, &form).await?;

    let normal_token = SensitiveString::from("normal_jwt".to_string());
    let form = LoginTokenCreateForm {
      token: normal_token.clone(),
      user_id: local_user.id,
      ip: None,
      user_agent: None,
      feed_only: false,
    };
    LoginToken::create(pool, &form).await?;

    let read = LoginToken::read_feed_only(pool, local_user.id).await?;
    assert_eq!(Some(feed_token.clone()), read.map(|t| t.token));

    // feed token validates with feed_only=true
    LoginToken::validate(pool, local_user.id, &feed_token, true).await?;
    let res = LoginToken::validate(pool, local_user.id, &feed_token, false).await;
    assert!(res.is_err());

    // normal token rejected with feed_only=true
    LoginToken::validate(pool, local_user.id, "normal_jwt", false).await?;
    let res = LoginToken::validate(pool, local_user.id, "normal_jwt", true).await;
    assert!(res.is_err());

    // list excludes feed-only token
    let logins = LoginToken::list(pool, local_user.id).await?;
    assert_eq!(1, logins.len());
    assert_eq!(normal_token, logins[0].token);

    Person::delete(pool, person.id).await?;
    Instance::delete(pool, instance.id).await?;
    Ok(())
  }
}
