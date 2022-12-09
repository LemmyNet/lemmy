use crate::{
  newtypes::{PersonId, ReviewCommentId},
  schema::review_comment::dsl::{approved, approver_id, review_comment, updated},
  source::review_comment::{ReviewComment, ReviewCommentForm},
  utils::{get_conn, naive_now, DbPool},
};
use diesel::{
  dsl::{insert_into, update},
  result::Error,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl ReviewComment {
  pub async fn create(pool: &DbPool, form: &ReviewCommentForm) -> Result<ReviewComment, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(review_comment)
      .values(form)
      .get_result(conn)
      .await
  }

  pub async fn approve(
    pool: &DbPool,
    review_id_: ReviewCommentId,
    by_approver_id: PersonId,
  ) -> Result<ReviewComment, Error> {
    let conn = &mut get_conn(pool).await?;
    update(review_comment.find(review_id_))
      .set((
        approved.eq(true),
        approver_id.eq(by_approver_id),
        updated.eq(naive_now()),
      ))
      .execute(conn)
      .await?;
    review_comment.find(review_id_).first::<Self>(conn).await
  }
}
