use crate::{
  newtypes::PersonId,
  schema::post_keyword_block,
  source::post_keyword_block::{PostKeywordBlock, PostKeywordBlockForm},
  utils::{get_conn, DbPool},
};
use diesel::{delete, insert_into, prelude::*, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

impl PostKeywordBlock {
  pub async fn for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> Result<Vec<PostKeywordBlock>, Error> {
    let conn = &mut get_conn(pool).await?;
    post_keyword_block::table
      .filter(post_keyword_block::person_id.eq(person_id))
      .load::<PostKeywordBlock>(conn)
      .await
  }

  pub async fn block_keyword(
    pool: &mut DbPool<'_>,
    post_keyword_block_form: &PostKeywordBlockForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_keyword_block::table)
      .values(post_keyword_block_form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn unblock_keyword(
    pool: &mut DbPool<'_>,
    post_keyword_block_form: &PostKeywordBlockForm,
  ) -> QueryResult<usize> {
    let conn = &mut get_conn(pool).await?;
    delete(post_keyword_block::table)
      .filter(post_keyword_block::person_id.eq(post_keyword_block_form.person_id))
      .filter(post_keyword_block::keyword.eq(&post_keyword_block_form.keyword))
      .execute(conn)
      .await
  }
}
