use crate::{
  schema::local_site::dsl::local_site,
  source::local_site::{LocalSite, LocalSiteInsertForm, LocalSiteUpdateForm},
  utils::DbConn,
};
use diesel::{dsl::insert_into, result::Error};
use diesel_async::RunQueryDsl;

impl LocalSite {
  pub async fn create(mut conn: impl DbConn, form: &LocalSiteInsertForm) -> Result<Self, Error> {
    insert_into(local_site)
      .values(form)
      .get_result::<Self>(&mut *conn)
      .await
  }
  pub async fn read(mut conn: impl DbConn) -> Result<Self, Error> {
    local_site.first::<Self>(&mut *conn).await
  }
  pub async fn update(mut conn: impl DbConn, form: &LocalSiteUpdateForm) -> Result<Self, Error> {
    diesel::update(local_site)
      .set(form)
      .get_result::<Self>(&mut *conn)
      .await
  }
  pub async fn delete(mut conn: impl DbConn) -> Result<usize, Error> {
    diesel::delete(local_site).execute(&mut *conn).await
  }
}
