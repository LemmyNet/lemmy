use crate::{
  newtypes::DbUrl,
  source::activity::{ReceivedActivity, SentActivity, SentActivityForm},
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

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
  pub async fn create(pool: &mut DbPool<'_>, ap_id_: DbUrl) -> Result<Self, Error> {
    use crate::schema::received_activity::dsl::{ap_id, received_activity};
    let conn = &mut get_conn(pool).await?;
    // TODO: use exists() first to avoid spamming log with conflicts
    insert_into(received_activity)
        .values(ap_id.eq(ap_id_))
        .get_result::<Self>(conn)
        .await
  }
}