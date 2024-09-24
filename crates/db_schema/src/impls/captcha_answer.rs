use crate::{
  schema::captcha_answer::dsl::{answer, captcha_answer},
  source::captcha_answer::{CaptchaAnswer, CaptchaAnswerForm, CheckCaptchaAnswer},
  utils::{functions::lower, get_conn, DbPool},
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
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

impl CaptchaAnswer {
  pub async fn insert(pool: &mut DbPool<'_>, captcha: &CaptchaAnswerForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    insert_into(captcha_answer)
      .values(captcha)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn check_captcha(
    pool: &mut DbPool<'_>,
    to_check: CheckCaptchaAnswer,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;

    // fetch requested captcha
    let captcha_exists =
      select(exists(captcha_answer.find(to_check.uuid).filter(
        lower(answer).eq(to_check.answer.to_lowercase().clone()),
      )))
      .get_result::<bool>(conn)
      .await?;

    // delete checked captcha
    delete(captcha_answer.find(to_check.uuid))
      .execute(conn)
      .await?;

    captcha_exists
      .then_some(())
      .ok_or(LemmyErrorType::CaptchaIncorrect.into())
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::{
    source::captcha_answer::{CaptchaAnswer, CaptchaAnswerForm, CheckCaptchaAnswer},
    utils::build_db_pool_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_captcha_happy_path() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted = CaptchaAnswer::insert(
      pool,
      &CaptchaAnswerForm {
        answer: "XYZ".to_string(),
      },
    )
    .await
    .expect("should not fail to insert captcha");

    let result = CaptchaAnswer::check_captcha(
      pool,
      CheckCaptchaAnswer {
        uuid: inserted.uuid,
        answer: "xyz".to_string(),
      },
    )
    .await;

    assert!(result.is_ok());
  }

  #[tokio::test]
  #[serial]
  async fn test_captcha_repeat_answer_fails() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted = CaptchaAnswer::insert(
      pool,
      &CaptchaAnswerForm {
        answer: "XYZ".to_string(),
      },
    )
    .await
    .expect("should not fail to insert captcha");

    let _result = CaptchaAnswer::check_captcha(
      pool,
      CheckCaptchaAnswer {
        uuid: inserted.uuid,
        answer: "xyz".to_string(),
      },
    )
    .await;

    let result_repeat = CaptchaAnswer::check_captcha(
      pool,
      CheckCaptchaAnswer {
        uuid: inserted.uuid,
        answer: "xyz".to_string(),
      },
    )
    .await;

    assert!(result_repeat.is_err());
  }
}
