use crate::{
  newtypes::{CommentId, NotificationId, PersonId},
  source::notification::{Notification, NotificationInsertForm},
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{insert_into, update},
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::notification;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Notification {
  pub async fn create(pool: &mut DbPool<'_>, form: &[NotificationInsertForm]) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(notification::table)
      .values(form)
      .on_conflict_do_nothing()
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreateNotification)
  }

  pub async fn read_by_comment_id(
    pool: &mut DbPool<'_>,
    comment_id: CommentId,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    notification::table
      .filter(notification::comment_id.eq(comment_id))
      .get_result::<Self>(conn)
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
    .with_lemmy_type(LemmyErrorType::CouldntUpdateNotification)
  }

  pub async fn mark_read_by_id_and_person(
    pool: &mut DbPool<'_>,
    notification_id: NotificationId,
    for_recipient_id: PersonId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(
      notification::table
        .filter(notification::id.eq(notification_id))
        .filter(notification::recipient_id.eq(for_recipient_id)),
    )
    .set(notification::read.eq(true))
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::NotFound)
  }
}
