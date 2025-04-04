use crate::{
  source::local_site_url_blocklist::{LocalSiteUrlBlocklist, LocalSiteUrlBlocklistForm},
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error};
use diesel_async::{
  scoped_futures::ScopedFutureExt,
  AsyncConnection,
  AsyncPgConnection,
  RunQueryDsl,
};
use lemmy_db_schema_file::schema::local_site_url_blocklist;

impl LocalSiteUrlBlocklist {
  pub async fn replace(pool: &mut DbPool<'_>, url_blocklist: Vec<String>) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;

    conn
      .transaction::<_, Error, _>(|conn| {
        async move {
          Self::clear(conn).await?;

          let forms = url_blocklist
            .into_iter()
            .map(|url| LocalSiteUrlBlocklistForm { url, updated: None })
            .collect::<Vec<_>>();

          insert_into(local_site_url_blocklist::table)
            .values(forms)
            .execute(conn)
            .await
        }
        .scope_boxed()
      })
      .await
  }

  async fn clear(conn: &mut AsyncPgConnection) -> Result<usize, Error> {
    diesel::delete(local_site_url_blocklist::table)
      .execute(conn)
      .await
  }

  pub async fn get_all(pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    local_site_url_blocklist::table
      .get_results::<Self>(conn)
      .await
  }
}
