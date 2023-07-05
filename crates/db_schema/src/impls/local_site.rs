use crate::{
  schema::local_site::dsl::local_site,
  source::local_site::{LocalSite, LocalSiteInsertForm, LocalSiteUpdateForm},
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error};
use diesel_async::RunQueryDsl;

impl LocalSite {
  pub async fn create(pool: &DbPool, form: &LocalSiteInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(local_site)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn read(pool: &DbPool) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    local_site.first::<Self>(conn).await
  }
  pub async fn update(pool: &DbPool, form: &LocalSiteUpdateForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_site)
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn delete(pool: &DbPool) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(local_site).execute(conn).await
  }
}
