use crate::{
  schema::local_site::dsl::local_site,
  source::local_site::{LocalSite, LocalSiteInsertForm, LocalSiteUpdateForm},
  utils::{DbPool, GetConn},
};
use diesel::{dsl::insert_into, result::Error};
use diesel_async::RunQueryDsl;

impl LocalSite {
  pub async fn create(
    mut pool: &mut impl GetConn,
    form: &LocalSiteInsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut *pool.get_conn().await?;
    insert_into(local_site)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn read(mut pool: &mut impl GetConn) -> Result<Self, Error> {
    let conn = &mut *pool.get_conn().await?;
    local_site.first::<Self>(conn).await
  }
  pub async fn update(
    mut pool: &mut impl GetConn,
    form: &LocalSiteUpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut *pool.get_conn().await?;
    diesel::update(local_site)
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn delete(mut pool: &mut impl GetConn) -> Result<usize, Error> {
    let conn = &mut *pool.get_conn().await?;
    diesel::delete(local_site).execute(conn).await
  }
}
