use crate::{
  newtypes::LocalUserId,
  source::keyword_block::{LocalUserKeywordBlock, LocalUserKeywordBlockForm},
};
use diesel::{ExpressionMethods, QueryDsl, delete, insert_into};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_db_schema_file::schema::local_user_keyword_block;
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl LocalUserKeywordBlock {
  pub async fn read(
    pool: &mut DbPool<'_>,
    for_local_user_id: LocalUserId,
  ) -> LemmyResult<Vec<String>> {
    let conn = &mut get_conn(pool).await?;
    local_user_keyword_block::table
      .filter(local_user_keyword_block::local_user_id.eq(for_local_user_id))
      .select(local_user_keyword_block::keyword)
      .load(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    blocking_keywords: Vec<String>,
    for_local_user_id: LocalUserId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    // No need to update if keywords unchanged
    conn
      .run_transaction(|conn| {
        async move {
          delete(local_user_keyword_block::table)
            .filter(local_user_keyword_block::local_user_id.eq(for_local_user_id))
            .filter(local_user_keyword_block::keyword.ne_all(&blocking_keywords))
            .execute(conn)
            .await
            .with_lemmy_type(LemmyErrorType::CouldntUpdate)?;
          let forms = blocking_keywords
            .into_iter()
            .map(|k| LocalUserKeywordBlockForm {
              local_user_id: for_local_user_id,
              keyword: k,
            })
            .collect::<Vec<_>>();
          insert_into(local_user_keyword_block::table)
            .values(forms)
            .on_conflict_do_nothing()
            .execute(conn)
            .await
            .with_lemmy_type(LemmyErrorType::CouldntUpdate)
        }
        .scope_boxed()
      })
      .await
  }
}
