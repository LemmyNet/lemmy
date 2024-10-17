use crate::{
  newtypes::{DbUrl, InboxId},
  schema::inbox,
  source::inbox::Inbox,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl Inbox {
  pub async fn read_or_create(pool: &mut DbPool<'_>, inbox: &DbUrl) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    insert_into(inbox::table)
      .values(inbox::url.eq(inbox))
      .on_conflict_do_nothing()
      .get_result::<Self>(conn)
      .await
  }

  pub async fn read(pool: &mut DbPool<'_>, id: InboxId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    inbox::table.find(id).first(conn).await
  }
}
