use crate::{
  schema::local_site_url_blocklist,
  source::local_site_url_blocklist::{LocalSiteUrlBlocklist, LocalSiteUrlBlocklistForm},
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

impl LocalSiteUrlBlocklist {
  pub async fn replace(pool: &mut DbPool<'_>, url_blocklist: Vec<String>) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;

    conn
      .build_transaction()
      .run(|conn| {
        Box::pin(async move {
          use crate::schema::local_site_url_blocklist::dsl::local_site_url_blocklist;

          Self::clear(conn).await?;

          let forms = url_blocklist
            .into_iter()
            .map(|url| LocalSiteUrlBlocklistForm { url, updated: None })
            .collect::<Vec<_>>();

          insert_into(local_site_url_blocklist)
            .values(forms)
            .execute(conn)
            .await?;

          Ok(())
        }) as _
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
