use crate::{
  newtypes::LocalUserId,
  schema::local_user_keyword_block,
  source::keyword_block::{LocalUserKeywordBlock, LocalUserKeywordBlockForm},
  utils::{get_conn, DbPool},
};
use diesel::{delete, insert_into, prelude::*, result::Error, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};

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
      .iter()
      .map(|keyword_block| keyword_block.keyword.clone())
      .collect();
    Ok(keywords)
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    blocking_keywords: Vec<String>,
    for_local_user_id: LocalUserId,
  ) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    // No need to update if keywords unchanged
    conn
      .transaction(|conn| {
        Box::pin(async move {
          let delete_old = delete(local_user_keyword_block::table)
            .filter(local_user_keyword_block::local_user_id.eq(for_local_user_id))
            .filter(local_user_keyword_block::keyword.ne_all(&blocking_keywords))
            .execute(conn);
          let forms = blocking_keywords
            .iter()
            .map(|k| LocalUserKeywordBlockForm {
              local_user_id: for_local_user_id,
              keyword: k.to_string(),
            })
            .collect::<Vec<_>>();
          let insert_new = insert_into(local_user_keyword_block::table)
            .values(forms)
            .on_conflict((
              local_user_keyword_block::keyword,
              local_user_keyword_block::local_user_id,
            ))
            .do_nothing()
            .execute(conn);
          tokio::try_join!(delete_old, insert_new)?;
          Ok(())
        }) as _
      })
      .await
  }
}
