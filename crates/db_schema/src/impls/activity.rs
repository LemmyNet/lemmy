use crate::{
  newtypes::DbUrl,
  source::activity::{ReceivedActivity, SentActivity, SentActivityForm},
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::*,
  result::{DatabaseErrorKind, Error, Error::DatabaseError},
  select,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};

impl SentActivity {
  pub async fn create(pool: &mut DbPool<'_>, form: SentActivityForm) -> Result<Self, Error> {
    use crate::schema::sent_activity::dsl::sent_activity;
    let conn = &mut get_conn(pool).await?;
    insert_into(sent_activity)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  pub async fn read_from_apub_id(pool: &mut DbPool<'_>, object_id: &DbUrl) -> Result<Self, Error> {
    use crate::schema::sent_activity::dsl::{ap_id, sent_activity};
    let conn = &mut get_conn(pool).await?;
    sent_activity
      .filter(ap_id.eq(object_id))
      .first::<Self>(conn)
      .await
  }
}

impl ReceivedActivity {
  pub async fn create(pool: &mut DbPool<'_>, ap_id_: &DbUrl) -> Result<Self, Error> {
    use crate::schema::received_activity::dsl::{ap_id, received_activity};
    let conn = &mut get_conn(pool).await?;
    conn
      .transaction(|conn| {
        async move {
          // Manually check if activity already exists, in order to avoid spamming logs with
          // insert conflict errors
          let exists = select(exists(received_activity.filter(ap_id.eq(ap_id_))))
            .get_result(conn)
            .await?;
          if exists {
            return Err(DatabaseError(
              DatabaseErrorKind::UniqueViolation,
              Box::new(String::new()),
            ));
          }
          insert_into(received_activity)
            .values(ap_id.eq(ap_id_))
            .get_result::<Self>(conn)
            .await
        }
        .scope_boxed()
      })
      .await
  }
}
