use super::actor_language::UNDETERMINED_ID;
use crate::{diesel::ExpressionMethods, newtypes::LanguageId, source::language::Language};
use diesel::{QueryDsl, dsl::count};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{language, post};
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::{
  CacheLock,
  build_cache,
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
};
use std::sync::LazyLock;

impl Language {
  /// Returns list of all available languages, with most used languages first
  pub async fn read_all(pool: &mut DbPool<'_>) -> LemmyResult<Vec<Self>> {
    static CACHE: CacheLock<Vec<Language>> = LazyLock::new(build_cache);
    CACHE
      .try_get_with((), async move {
        let conn = &mut get_conn(pool).await?;
        language::table
          .left_join(post::table)
          .group_by(language::id)
          .order_by(count(post::id).desc())
          .select(language::all_columns)
          .load(conn)
          .await
      })
      .await
      .map_err(|_e| LemmyErrorType::NotFound.into())
  }

  pub async fn read_from_id(pool: &mut DbPool<'_>, id_: LanguageId) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    language::table
      .find(id_)
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Attempts to find the given language code and return its ID.
  pub async fn read_id_from_code(pool: &mut DbPool<'_>, code_: &str) -> LemmyResult<LanguageId> {
    let conn = &mut get_conn(pool).await?;
    let res = language::table
      .filter(language::code.eq(code_))
      .first::<Self>(conn)
      .await
      .map(|l| l.id);

    // Return undetermined by default
    Ok(res.unwrap_or(UNDETERMINED_ID))
  }
}

#[cfg(test)]
#[expect(clippy::indexing_slicing)]
mod tests {

  use crate::source::language::Language;
  use lemmy_diesel_utils::connection::build_db_pool_for_tests;
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_languages() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let mut all = Language::read_all(pool).await?;

    // Languages are returned in order of popularity, so to make this test work we need to
    // manually sort them by id.
    all.sort_by(|a, b| a.id.0.cmp(&b.id.0));

    assert_eq!(184, all.len());
    assert_eq!("ak", all[5].code);
    assert_eq!("lv", all[99].code);
    assert_eq!("yi", all[179].code);

    Ok(())
  }
}
