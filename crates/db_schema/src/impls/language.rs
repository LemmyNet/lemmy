use crate::{
  diesel::ExpressionMethods,
  newtypes::LanguageId,
  schema::language::dsl::{code, id, language},
  source::language::Language,
  utils::{get_conn, DbPool},
};
use diesel::{result::Error, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

impl Language {
  pub async fn read_all(pool: &DbPool) -> Result<Vec<Language>, Error> {
    let conn = &mut get_conn(pool).await?;
    Self::read_all_conn(conn).await
  }

  pub async fn read_all_conn(conn: &mut AsyncPgConnection) -> Result<Vec<Language>, Error> {
    language.load::<Self>(conn).await
  }

  pub async fn read_from_id(pool: &DbPool, id_: LanguageId) -> Result<Language, Error> {
    let conn = &mut get_conn(pool).await?;
    language.filter(id.eq(id_)).first::<Self>(conn).await
  }

  pub async fn read_id_from_code(pool: &DbPool, code_: &str) -> Result<LanguageId, Error> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      language
        .filter(code.eq(code_))
        .first::<Self>(conn)
        .await?
        .id,
    )
  }

  pub async fn read_id_from_code_opt(
    pool: &DbPool,
    code_: Option<&str>,
  ) -> Result<Option<LanguageId>, Error> {
    if let Some(code_) = code_ {
      Ok(Some(Language::read_id_from_code(pool, code_).await?))
    } else {
      Ok(None)
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::{source::language::Language, utils::build_db_pool_for_tests};
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_languages() {
    let pool = &build_db_pool_for_tests().await;

    let all = Language::read_all(pool).await.unwrap();

    assert_eq!(184, all.len());
    assert_eq!("ak", all[5].code);
    assert_eq!("lv", all[99].code);
    assert_eq!("yi", all[179].code);
  }
}
