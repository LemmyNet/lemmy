use crate::{
  newtypes::{CommentId, LocalUserId, NotificationId},
  source::notification::{
    LocalUserNotification,
    LocalUserNotificationInsertForm,
    Notification,
    NotificationInsertForm,
  },
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{insert_into, update},
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{local_user_notification, notification};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Notification {
  pub async fn create(pool: &mut DbPool<'_>, form: &NotificationInsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(notification::table)
      .values(form)
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
}

impl LocalUserNotification {
  pub async fn create(
    pool: &mut DbPool<'_>,
    form: &LocalUserNotificationInsertForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    // since the return here isnt utilized, we dont need to do an update
    // but get_result doesn't return the existing row here
    insert_into(local_user_notification::table)
      .values(form)
      .on_conflict_do_nothing()
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreateNotification)
  }

  pub async fn mark_all_as_read(
    pool: &mut DbPool<'_>,
    for_recipient_id: LocalUserId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      local_user_notification::table
        .filter(local_user_notification::recipient_id.eq(for_recipient_id))
        .filter(local_user_notification::read.eq(false)),
    )
    .set(local_user_notification::read.eq(true))
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateNotification)
  }

  pub async fn mark_read_by_id_and_person(
    pool: &mut DbPool<'_>,
    notification_id: NotificationId,
    for_recipient_id: LocalUserId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    update(
      local_user_notification::table
        .filter(local_user_notification::notification_id.eq(notification_id))
        .filter(local_user_notification::recipient_id.eq(for_recipient_id)),
    )
    .set(local_user_notification::read.eq(true))
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::NotFound)
  }
}
