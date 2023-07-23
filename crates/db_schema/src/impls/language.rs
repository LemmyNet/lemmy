use crate::{
  diesel::ExpressionMethods,
  newtypes::LanguageId,
  schema::language::dsl::{code, id, language},
  source::language::Language,
  utils::{get_conn, DbPool},
};
use diesel::{result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

impl Language {
  pub async fn read_all(pool: &mut DbPool<'_>) -> Result<Vec<Language>, Error> {
    let conn = &mut get_conn(pool).await?;
    language.load::<Self>(conn).await
  }

  pub async fn read_from_id(pool: &mut DbPool<'_>, id_: LanguageId) -> Result<Language, Error> {
    let conn = &mut get_conn(pool).await?;
    language.filter(id.eq(id_)).first::<Self>(conn).await
  }

  /// Attempts to find the given language code and return its ID. If not found, returns none.
  pub async fn read_id_from_code(
    pool: &mut DbPool<'_>,
    code_: Option<&str>,
  ) -> Result<Option<LanguageId>, Error> {
    if let Some(code_) = code_ {
      let conn = &mut get_conn(pool).await?;
      Ok(
        language
          .filter(code.eq(code_))
          .first::<Self>(conn)
          .await
          .map(|l| l.id)
          .ok(),
      )
    } else {
      Ok(None)
    }
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{source::language::Language, utils::build_db_pool_for_tests};
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_languages() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let all = Language::read_all(pool).await.unwrap();

    assert_eq!(184, all.len());
    assert_eq!("ak", all[5].code);
    assert_eq!("lv", all[99].code);
    assert_eq!("yi", all[179].code);
  }
}
