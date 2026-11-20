use crate::source::email_verification::{EmailVerification, EmailVerificationForm};
use diesel::{ExpressionMethods, QueryDsl, dsl::IntervalDsl, insert_into};
use diesel_async::{AsyncConnection, RunQueryDsl};
use lemmy_db_schema_file::{
  newtypes::LocalUserId,
  schema::{email_verification, local_user},
};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  utils::{now, seconds_to_pg_interval},
};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType, LemmyResult};

impl EmailVerification {
  pub async fn create(pool: &mut DbPool<'_>, form: &EmailVerificationForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(email_verification::table)
      .values(form)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  /// Insert a verification token unless the account already received `max_requests` tokens
  /// within the last `interval_seconds`, or its email was verified in the meantime. The account
  /// row is locked for the duration of the transaction, so concurrent requests from any number
  /// of backend processes are serialized, and no token is left behind by a resend that races
  /// with the verification.
  pub async fn create_with_account_limit(
    pool: &mut DbPool<'_>,
    form: &EmailVerificationForm,
    max_requests: i32,
    interval_seconds: i32,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    conn
      .transaction::<Option<Self>, LemmyError, _>(async |conn| {
        let email_verified: bool = local_user::table
          .find(form.local_user_id)
          .select(local_user::email_verified)
          .for_update()
          .first(conn)
          .await
          .with_lemmy_type(LemmyErrorType::NotFound)?;
        if email_verified {
          return Ok(None);
        }

        let recent: i64 = email_verification::table
          .filter(email_verification::local_user_id.eq(form.local_user_id))
          .filter(
            email_verification::published_at.gt(now() - seconds_to_pg_interval(interval_seconds)),
          )
          .count()
          .get_result(conn)
          .await?;
        if recent >= i64::from(max_requests) {
          return Ok(None);
        }

        insert_into(email_verification::table)
          .values(form)
          .get_result(conn)
          .await
          .map(Some)
          .with_lemmy_type(LemmyErrorType::CouldntCreate)
      })
      .await
  }

  pub async fn read_for_token(pool: &mut DbPool<'_>, token: &str) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    email_verification::table
      .filter(email_verification::verification_token.eq(token))
      .filter(email_verification::published_at.gt(now() - 7.days()))
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
  pub async fn delete_old_tokens_for_local_user(
    pool: &mut DbPool<'_>,
    local_user_id_: LocalUserId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      email_verification::table.filter(email_verification::local_user_id.eq(local_user_id_)),
    )
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::Deleted)
  }
}

#[cfg(test)]
mod tests {
  use crate::source::{
    email_verification::{EmailVerification, EmailVerificationForm},
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
    person::{Person, PersonInsertForm},
  };
  use lemmy_db_schema_file::newtypes::LocalUserId;
  use lemmy_diesel_utils::{
    connection::{DbPool, build_db_pool_for_tests},
    traits::Crud,
  };
  use lemmy_utils::error::{LemmyErrorType, LemmyResult};
  use serial_test::serial;

  static TOKEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

  fn form(local_user_id: LocalUserId) -> EmailVerificationForm {
    let n = TOKEN.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    EmailVerificationForm {
      local_user_id,
      email: "limit@example.test".into(),
      verification_token: format!("limit-token-{n}"),
    }
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
  #[serial]
  async fn test_account_limit_is_shared_and_atomic() -> LemmyResult<()> {
    // Two independent pools stand in for two backend processes.
    let pool_a = build_db_pool_for_tests();
    let pool_b = build_db_pool_for_tests();
    let pool = &mut (&pool_a).into();

    let instance = Instance::read_or_create(pool, "limit.tld").await?;
    let person = Person::create(
      pool,
      &PersonInsertForm::test_form(instance.id, "limit_user"),
    )
    .await?;
    let user = LocalUser::create(pool, &LocalUserInsertForm::test_form(person.id), vec![]).await?;
    let person2 = Person::create(
      pool,
      &PersonInsertForm::test_form(instance.id, "limit_user2"),
    )
    .await?;
    let user2 =
      LocalUser::create(pool, &LocalUserInsertForm::test_form(person2.id), vec![]).await?;

    // 12 concurrent requests, alternating between both "processes", limit 2 per hour.
    let mut handles = vec![];
    for i in 0..12 {
      let p = if i % 2 == 0 {
        pool_a.clone()
      } else {
        pool_b.clone()
      };
      let f = form(user.id);
      handles.push(tokio::spawn(async move {
        EmailVerification::create_with_account_limit(&mut (&p).into(), &f, 2, 3600).await
      }));
    }
    let mut allowed = 0;
    for h in handles {
      if h.await??.is_some() {
        allowed += 1;
      }
    }
    assert_eq!(2, allowed);

    // Limit survives a "restart": a brand-new pool still sees the limit.
    let pool_c = build_db_pool_for_tests();
    let res =
      EmailVerification::create_with_account_limit(&mut (&pool_c).into(), &form(user.id), 2, 3600)
        .await?;
    assert!(res.is_none());

    // Other accounts are unaffected.
    let res = EmailVerification::create_with_account_limit(pool, &form(user2.id), 2, 3600).await?;
    assert!(res.is_some());

    // Window expiry: age the existing rows, then a new token is allowed again.
    {
      use diesel::{ExpressionMethods, QueryDsl, dsl::IntervalDsl};
      use diesel_async::RunQueryDsl;
      use lemmy_db_schema_file::schema::email_verification;
      let conn = &mut lemmy_diesel_utils::connection::get_conn(pool).await?;
      diesel::update(
        email_verification::table.filter(email_verification::local_user_id.eq(user.id)),
      )
      .set(email_verification::published_at.eq(lemmy_diesel_utils::utils::now() - 2.hours()))
      .execute(conn)
      .await?;
    }
    let res = EmailVerification::create_with_account_limit(pool, &form(user.id), 2, 3600).await?;
    assert!(res.is_some());

    // max_requests = 0 disables sending.
    let res = EmailVerification::create_with_account_limit(pool, &form(user2.id), 0, 3600).await?;
    assert!(res.is_none());

    Instance::delete(pool, instance.id).await?;
    Ok(())
  }

  async fn count(pool: &mut DbPool<'_>, local_user_id: LocalUserId) -> LemmyResult<i64> {
    use diesel::{ExpressionMethods, QueryDsl};
    use diesel_async::RunQueryDsl;
    use lemmy_db_schema_file::schema::email_verification;
    let conn = &mut lemmy_diesel_utils::connection::get_conn(pool).await?;
    Ok(
      email_verification::table
        .filter(email_verification::local_user_id.eq(local_user_id))
        .count()
        .get_result(conn)
        .await?,
    )
  }

  #[tokio::test]
  #[serial]
  async fn test_verification_removes_all_tokens_of_account() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let instance = Instance::read_or_create(pool, "cleanup.tld").await?;
    let person = Person::create(
      pool,
      &PersonInsertForm::test_form(instance.id, "cleanup_user"),
    )
    .await?;
    let user = LocalUser::create(pool, &LocalUserInsertForm::test_form(person.id), vec![]).await?;
    let person2 = Person::create(
      pool,
      &PersonInsertForm::test_form(instance.id, "cleanup_user2"),
    )
    .await?;
    let user2 =
      LocalUser::create(pool, &LocalUserInsertForm::test_form(person2.id), vec![]).await?;
    let other = EmailVerification::create(pool, &form(user2.id)).await?;

    // Registration token plus resends up to the account limit of 4.
    let mut tokens = vec![EmailVerification::create(pool, &form(user.id)).await?];
    while let Some(token) =
      EmailVerification::create_with_account_limit(pool, &form(user.id), 4, 3600).await?
    {
      tokens.push(token);
    }
    assert_eq!(4, tokens.len());

    // Redeem an older token, like verify_email does: mark verified, then delete all tokens.
    let older = tokens.get(1).map(|t| t.verification_token.clone());
    let redeemed = EmailVerification::read_for_token(pool, &older.unwrap_or_default()).await?;
    assert_eq!(user.id, redeemed.local_user_id);
    let form_verified = LocalUserUpdateForm {
      email_verified: Some(true),
      ..Default::default()
    };
    LocalUser::update(pool, user.id, &form_verified).await?;
    let deleted = EmailVerification::delete_old_tokens_for_local_user(pool, user.id).await?;
    assert_eq!(4, deleted);

    assert_eq!(0, count(pool, user.id).await?);
    for token in &tokens {
      let res = EmailVerification::read_for_token(pool, &token.verification_token).await;
      assert!(
        res.is_err_and(|e| e.error_type == LemmyErrorType::NotFound),
        "redeemed account must not keep valid tokens"
      );
    }
    // Tokens of other accounts are untouched.
    assert_eq!(1, count(pool, user2.id).await?);
    assert!(
      EmailVerification::read_for_token(pool, &other.verification_token)
        .await
        .is_ok()
    );

    // A resend that read the account before it was verified must not leave a token behind.
    let res = EmailVerification::create_with_account_limit(pool, &form(user.id), 4, 3600).await?;
    assert!(res.is_none());
    assert_eq!(0, count(pool, user.id).await?);

    // While unverified, deleting the tokens frees the account limit again.
    let form_unverified = LocalUserUpdateForm {
      email_verified: Some(false),
      ..Default::default()
    };
    LocalUser::update(pool, user.id, &form_unverified).await?;
    let res = EmailVerification::create_with_account_limit(pool, &form(user.id), 4, 3600).await?;
    assert!(res.is_some());
    assert_eq!(1, count(pool, user.id).await?);

    Instance::delete(pool, instance.id).await?;
    Ok(())
  }
}
