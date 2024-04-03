use crate::{
  schema::local_site::dsl::local_site,
  source::local_site::{LocalSite, LocalSiteInsertForm, LocalSiteUpdateForm},
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error};
use diesel_async::RunQueryDsl;
use lemmy_utils::{error::LemmyError, CACHE_DURATION_SHORT};
use moka::future::Cache;
use once_cell::sync::Lazy;

impl LocalSite {
  pub async fn create(pool: &mut DbPool<'_>, form: &LocalSiteInsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(local_site)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn read(pool: &mut DbPool<'_>) -> Result<Self, LemmyError> {
    static CACHE: Lazy<Cache<(), LocalSite>> = Lazy::new(|| {
      Cache::builder()
        .max_capacity(1)
        .time_to_live(CACHE_DURATION_SHORT)
        .build()
    });
    Ok(
      CACHE
        .try_get_with((), async {
          let conn = &mut get_conn(pool).await?;
          local_site.first::<Self>(conn).await
        })
        .await?,
    )
  }
  pub async fn update(pool: &mut DbPool<'_>, form: &LocalSiteUpdateForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(local_site)
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn delete(pool: &mut DbPool<'_>) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(local_site).execute(conn).await
  }
}
