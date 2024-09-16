use crate::{
  diesel::ExpressionMethods,
  newtypes::LanguageId,
  schema::language,
  source::language::Language,
  utils::{get_conn, DbPool},
};
use diesel::{result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

impl Language {
  pub async fn read_all(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    language::table.load(conn).await
  }

  pub async fn read_from_id(pool: &mut DbPool<'_>, id_: LanguageId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    language::table.find(id_).first(conn).await
  }

  /// Attempts to find the given language code and return its ID. If not found, returns none.
  pub async fn read_id_from_code(
    pool: &mut DbPool<'_>,
    code_: Option<&str>,
  ) -> Result<Option<LanguageId>, Error> {
    if let Some(code_) = code_ {
      let conn = &mut get_conn(pool).await?;
      Ok(
        language::table
          .filter(language::code.eq(code_))
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
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::{source::language::Language, utils::build_db_pool_for_tests};
  use pretty_assertions::assert_eq;
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
