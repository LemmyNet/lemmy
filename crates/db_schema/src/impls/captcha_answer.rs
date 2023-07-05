use crate::{
  schema::captcha_answer::dsl::{answer, captcha_answer, uuid},
  source::captcha_answer::{CaptchaAnswer, CaptchaAnswerForm, CheckCaptchaAnswer},
  utils::{functions::lower, GetConn},
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
use lemmy_db_schema::utils::RunQueryDsl;

impl CaptchaAnswer {
  pub async fn insert(mut conn: impl GetConn, captcha: &CaptchaAnswerForm) -> Result<Self, Error> {
    insert_into(captcha_answer)
      .values(captcha)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn check_captcha(
    mut conn: impl GetConn,
    to_check: CheckCaptchaAnswer,
  ) -> Result<bool, Error> {
    // fetch requested captcha
    let captcha_exists = select(exists(
      captcha_answer
        .filter((uuid).eq(to_check.uuid))
        .filter(lower(answer).eq(to_check.answer.to_lowercase().clone())),
    ))
    .get_result::<bool>(conn)
    .await?;

    // delete checked captcha
    delete(captcha_answer.filter(uuid.eq(to_check.uuid)))
      .execute(conn)
      .await?;

    Ok(captcha_exists)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::captcha_answer::{CaptchaAnswer, CaptchaAnswerForm, CheckCaptchaAnswer},
    utils::build_db_conn_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_captcha_happy_path() {
    let mut conn = build_db_conn_for_tests().await;

    let inserted = CaptchaAnswer::insert(
      conn,
      &CaptchaAnswerForm {
        answer: "XYZ".to_string(),
      },
    )
    .await
    .expect("should not fail to insert captcha");

    let result = CaptchaAnswer::check_captcha(
      conn,
      CheckCaptchaAnswer {
        uuid: inserted.uuid,
        answer: "xyz".to_string(),
      },
    )
    .await;

    assert!(result.is_ok());
    assert!(result.unwrap());
  }

  #[tokio::test]
  #[serial]
  async fn test_captcha_repeat_answer_fails() {
    let mut conn = build_db_conn_for_tests().await;

    let inserted = CaptchaAnswer::insert(
      conn,
      &CaptchaAnswerForm {
        answer: "XYZ".to_string(),
      },
    )
    .await
    .expect("should not fail to insert captcha");

    let _result = CaptchaAnswer::check_captcha(
      conn,
      CheckCaptchaAnswer {
        uuid: inserted.uuid,
        answer: "xyz".to_string(),
      },
    )
    .await;

    let result_repeat = CaptchaAnswer::check_captcha(
      conn,
      CheckCaptchaAnswer {
        uuid: inserted.uuid,
        answer: "xyz".to_string(),
      },
    )
    .await;

    assert!(result_repeat.is_ok());
    assert!(!result_repeat.unwrap());
  }
}
