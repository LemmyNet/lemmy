use crate::{
  schema::local_site_url_blocklist::dsl::{local_site_url_blocklist, url},
  source::local_site_url_blocklist::{LocalSiteUrlBlocklist, LocalSiteUrlBlocklistForm},
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::{delete, insert_into},
  result::Error,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;

impl LocalSiteUrlBlocklist {
  pub async fn add(pool: &mut DbPool<'_>, insert_url: String) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(local_site_url_blocklist)
      .values(&LocalSiteUrlBlocklistForm { url: insert_url })
      .execute(conn)
      .await?;

    Ok(())
  }

  pub async fn remove(pool: &mut DbPool<'_>, remove_url: String) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    delete(local_site_url_blocklist.filter(url.eq(remove_url)))
      .execute(conn)
      .await?;
    Ok(())
  }

  pub async fn get_all(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    Ok(local_site_url_blocklist.get_results::<Self>(conn).await?)
  }
}
