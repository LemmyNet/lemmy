use crate::{
  schema::local_site_rate_limit,
  source::local_site_rate_limit::{
    LocalSiteRateLimit,
    LocalSiteRateLimitInsertForm,
    LocalSiteRateLimitUpdateForm,
  },
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error};
use diesel_async::RunQueryDsl;

impl LocalSiteRateLimit {
  pub async fn read(pool: &DbPool) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    local_site_rate_limit::table.first::<Self>(conn).await
  }

  pub async fn create(pool: &DbPool, form: &LocalSiteRateLimitInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(local_site_rate_limit::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn update(pool: &DbPool, form: &LocalSiteRateLimitUpdateForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_site_rate_limit::table)
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}
