use crate::{
  schema::local_site::dsl::local_site,
  source::local_site::{LocalSite, LocalSiteInsertForm, LocalSiteUpdateForm},
  utils::{DbPool, DbPoolRef, RunQueryDsl},
};
use diesel::{dsl::insert_into, result::Error};

impl LocalSite {
  pub async fn create(pool: DbPoolRef<'_>, form: &LocalSiteInsertForm) -> Result<Self, Error> {
    let conn = pool;
    insert_into(local_site)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn read(pool: DbPoolRef<'_>) -> Result<Self, Error> {
    let conn = pool;
    local_site.first::<Self>(conn).await
  }
  pub async fn update(pool: DbPoolRef<'_>, form: &LocalSiteUpdateForm) -> Result<Self, Error> {
    let conn = pool;
    diesel::update(local_site)
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn delete(pool: DbPoolRef<'_>) -> Result<usize, Error> {
    let conn = pool;
    diesel::delete(local_site).execute(conn).await
  }
}
