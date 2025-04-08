use crate::{
  newtypes::LocalUserId,
  source::keyword_block::{LocalUserKeywordBlock, LocalUserKeywordBlockForm},
  utils::{get_conn, DbPool},
};
use diesel::{delete, insert_into, prelude::*, result::Error, QueryDsl};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::local_user_keyword_block;

impl LocalUserKeywordBlock {
  pub async fn read(
    pool: &mut DbPool<'_>,
    for_local_user_id: LocalUserId,
  ) -> Result<Vec<String>, Error> {
    let conn = &mut get_conn(pool).await?;
    let keyword_blocks = local_user_keyword_block::table
      .filter(local_user_keyword_block::local_user_id.eq(for_local_user_id))
      .load::<LocalUserKeywordBlock>(conn)
      .await?;
    let keywords = keyword_blocks
      .into_iter()
      .map(|keyword_block| keyword_block.keyword)
      .collect();
    Ok(keywords)
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    blocking_keywords: Vec<String>,
    for_local_user_id: LocalUserId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    // No need to update if keywords unchanged
    conn
      .transaction::<_, Error, _>(|conn| {
        async move {
          delete(local_user_keyword_block::table)
            .filter(local_user_keyword_block::local_user_id.eq(for_local_user_id))
            .filter(local_user_keyword_block::keyword.ne_all(&blocking_keywords))
            .execute(conn)
            .await?;
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
        }
        .scope_boxed()
      })
      .await
  }
}
