use crate::{
  schema::captcha_answer,
  source::captcha_answer::CaptchaAnswer,
  utils::{functions::lower, get_conn, naive_now, DbPool},
};
use diesel::{
  delete,
  dsl::exists,
  insert_into,
  result::Error,
  select,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl CaptchaAnswer {
  pub async fn insert(pool: &DbPool, captcha: &CaptchaAnswer) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    insert_into(captcha_answer::table)
      .values(captcha)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn check_captcha(pool: &DbPool, to_check: CaptchaAnswer) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;

    // delete any expired captchas
    delete(captcha_answer::table.filter(captcha_answer::expires.lt(&naive_now())))
      .execute(conn)
      .await?;

    // fetch requested captcha
    let captcha_exists = select(exists(
      captcha_answer::dsl::captcha_answer
        .filter((captcha_answer::dsl::uuid).eq(to_check.uuid.clone()))
        .filter(lower(captcha_answer::dsl::answer).eq(to_check.answer.to_lowercase().clone())),
    ))
    .get_result::<bool>(conn)
    .await?;

    // delete checked captcha
    delete(captcha_answer::table.filter(captcha_answer::uuid.eq(to_check.uuid.clone())))
      .execute(conn)
      .await?;

    Ok(captcha_exists)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::captcha_answer::CaptchaAnswer,
    utils::{build_db_pool_for_tests, naive_now},
  };
  use chrono::Duration;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_captcha_happy_path() {
    let pool = &build_db_pool_for_tests().await;

    let captcha_a_id = "a".to_string();

    let _ = CaptchaAnswer::insert(
      pool,
      &CaptchaAnswer {
        uuid: captcha_a_id.clone(),
        answer: "XYZ".to_string(),
        expires: naive_now() + Duration::minutes(10),
      },
    )
    .await;

    let result = CaptchaAnswer::check_captcha(
      pool,
      CaptchaAnswer {
        uuid: captcha_a_id.clone(),
        answer: "xyz".to_string(),
        expires: chrono::NaiveDateTime::MIN,
      },
    )
    .await;

    assert!(result.is_ok());
    assert!(result.unwrap());
  }

  #[tokio::test]
  #[serial]
  async fn test_captcha_repeat_answer_fails() {
    let pool = &build_db_pool_for_tests().await;

    let captcha_a_id = "a".to_string();

    let _ = CaptchaAnswer::insert(
      pool,
      &CaptchaAnswer {
        uuid: captcha_a_id.clone(),
        answer: "XYZ".to_string(),
        expires: naive_now() + Duration::minutes(10),
      },
    )
    .await;

    let result = CaptchaAnswer::check_captcha(
      pool,
      CaptchaAnswer {
        uuid: captcha_a_id.clone(),
        answer: "xyz".to_string(),
        expires: chrono::NaiveDateTime::MIN,
      },
    )
    .await;

    let result_repeat = CaptchaAnswer::check_captcha(
      pool,
      CaptchaAnswer {
        uuid: captcha_a_id.clone(),
        answer: "xyz".to_string(),
        expires: chrono::NaiveDateTime::MIN,
      },
    )
    .await;

    assert!(result_repeat.is_ok());
    assert!(!result_repeat.unwrap());
  }

  #[tokio::test]
  #[serial]
  async fn test_captcha_expired_fails() {
    let pool = &build_db_pool_for_tests().await;

    let expired_id = "already_expired".to_string();

    let _ = CaptchaAnswer::insert(
      pool,
      &CaptchaAnswer {
        uuid: expired_id.clone(),
        answer: "xyz".to_string(),
        expires: naive_now() - Duration::seconds(1),
      },
    )
    .await;

    let expired_result = CaptchaAnswer::check_captcha(
      pool,
      CaptchaAnswer {
        uuid: expired_id.clone(),
        answer: "xyz".to_string(),
        expires: chrono::NaiveDateTime::MIN,
      },
    )
    .await;

    assert!(expired_result.is_ok());
    assert!(!expired_result.unwrap());
  }
}
