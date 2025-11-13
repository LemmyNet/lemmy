use crate::{
  newtypes::{CommentId, NotificationId, PostId},
  source::notification::{Notification, NotificationInsertForm},
};
use diesel::{
  ExpressionMethods,
  QueryDsl,
  delete,
  dsl::{insert_into, update},
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::{PersonId, schema::notification};
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Notification {
  pub async fn create(
    pool: &mut DbPool<'_>,
    form: &[NotificationInsertForm],
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    insert_into(notification::table)
      .values(form)
      .on_conflict_do_nothing()
      .get_results::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  pub async fn mark_read_by_comment_and_recipient(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
    recipient_id: PersonId,
    read: bool,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(
      notification::table
        .filter(notification::comment_id.eq(comment_id))
        .filter(notification::recipient_id.eq(recipient_id)),
    )
    .set(notification::read.eq(read))
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn mark_read_by_post_and_recipient(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    recipient_id: PersonId,
    read: bool,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(
      notification::table
        .filter(notification::post_id.eq(post_id))
        .filter(notification::recipient_id.eq(recipient_id)),
    )
    .set(notification::read.eq(read))
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn mark_all_as_read(
    pool: &mut DbPool<'_>,
    for_recipient_id: PersonId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      notification::table
        .filter(notification::recipient_id.eq(for_recipient_id))
        .filter(notification::read.eq(false)),
    )
    .set(notification::read.eq(true))
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }

  pub async fn mark_read_by_id_and_person(
    pool: &mut DbPool<'_>,
    notification_id: NotificationId,
    recipient_id: PersonId,
    read: bool,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(
      notification::table
        .filter(notification::id.eq(notification_id))
        .filter(notification::recipient_id.eq(recipient_id)),
    )
    .set(notification::read.eq(read))
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::NotFound)
  }

  /// Only for tests
  pub async fn delete(pool: &mut DbPool<'_>, id: NotificationId) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    delete(notification::table.filter(notification::id.eq(id)))
      .execute(conn)
      .await?;
    Ok(())
  }
}
