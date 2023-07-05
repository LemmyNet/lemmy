use crate::{
  schema::local_site::dsl::local_site,
  source::local_site::{LocalSite, LocalSiteInsertForm, LocalSiteUpdateForm},
  utils::GetConn,
};
use diesel::{dsl::insert_into, result::Error};
use lemmy_db_schema::utils::RunQueryDsl;

impl LocalSite {
  pub async fn create(mut conn: impl GetConn, form: &LocalSiteInsertForm) -> Result<Self, Error> {
    insert_into(local_site)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn read(mut conn: impl GetConn) -> Result<Self, Error> {
    local_site.first::<Self>(conn).await
  }
  pub async fn update(mut conn: impl GetConn, form: &LocalSiteUpdateForm) -> Result<Self, Error> {
    diesel::update(local_site)
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn delete(mut conn: impl GetConn) -> Result<usize, Error> {
    diesel::delete(local_site).execute(conn).await
  }
}
