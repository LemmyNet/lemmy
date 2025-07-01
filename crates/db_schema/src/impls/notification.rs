use crate::{
  newtypes::{CommentId, LocalUserId, NotificationId},
  source::notification::{
    Notification,
    NotificationInsertForm,
    PersonNotification,
    PersonNotificationInsertForm,
  },
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{insert_into, update},
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{notification, person_notification};
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

impl PersonNotification {
  pub async fn create(
    pool: &mut DbPool<'_>,
    form: &PersonNotificationInsertForm,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    insert_into(person_notification::table)
      .values(form)
      .on_conflict_do_nothing()
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreateNotification)
  }

  pub async fn mark_all_as_read(
    pool: &mut DbPool<'_>,
    for_recipient_id: LocalUserId,
  ) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      person_notification::table
        .filter(person_notification::recipient_id.eq(for_recipient_id))
        .filter(person_notification::read.eq(false)),
    )
    .set(person_notification::read.eq(true))
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
      person_notification::table
        .filter(person_notification::notification_id.eq(notification_id))
        .filter(person_notification::recipient_id.eq(for_recipient_id)),
    )
    .set(person_notification::read.eq(true))
    .execute(conn)
    .await
    .with_lemmy_type(LemmyErrorType::NotFound)
  }
}
