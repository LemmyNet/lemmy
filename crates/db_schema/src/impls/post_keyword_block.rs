use crate::{
  newtypes::PersonId,
  schema::user_post_keyword_block::dsl::{keyword, person_id, user_post_keyword_block},
  source::user_post_keyword_block::{UserPostKeywordBlock, UserPostKeywordBlockForm},
  utils::{get_conn, DbPool},
};
use diesel::{delete, insert_into, prelude::*, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;

impl UserPostKeywordBlock {
  pub async fn for_person(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
  ) -> Result<Vec<UserPostKeywordBlock>, Error> {
    let conn = &mut get_conn(pool).await?;
    user_post_keyword_block
      .filter(person_id.eq(for_person_id))
      .load::<UserPostKeywordBlock>(conn)
      .await
  }

  pub async fn update(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
    keywords_to_block_posts: Vec<String>,
  ) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    // No need to update if keywords unchanged
    let current = UserPostKeywordBlock::for_person(&mut conn.into(), for_person_id).await?;
    if current
      .iter()
      .map(|obj| obj.keyword.clone())
      .collect::<Vec<_>>()
      == keywords_to_block_posts
    {
      return Ok(());
    }
    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          let delete_old = delete(user_post_keyword_block)
            .filter(person_id.eq(for_person_id))
            .filter(keyword.ne_all(&keywords_to_block_posts))
            .execute(conn);
          let forms = keywords_to_block_posts
            .iter()
            .map(|k| UserPostKeywordBlockForm {
              person_id: for_person_id,
              keyword: k.clone(),
            })
            .collect::<Vec<_>>();
          let insert_new = insert_into(user_post_keyword_block)
            .values(forms)
            .on_conflict((keyword, person_id))
            .do_nothing()
            .execute(conn);
          tokio::try_join!(delete_old, insert_new)?;
          Ok(())
        }) as _
      })
      .await
  }

  pub async fn block_keyword(
    pool: &mut DbPool<'_>,
    post_keyword_block_form: &UserPostKeywordBlockForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(user_post_keyword_block)
      .values(post_keyword_block_form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn unblock_keyword(
    pool: &mut DbPool<'_>,
    post_keyword_block_form: &UserPostKeywordBlockForm,
  ) -> QueryResult<usize> {
    let conn = &mut get_conn(pool).await?;
    delete(user_post_keyword_block)
      .filter(person_id.eq(post_keyword_block_form.person_id))
      .filter(keyword.eq(&post_keyword_block_form.keyword))
      .execute(conn)
      .await
  }
}
