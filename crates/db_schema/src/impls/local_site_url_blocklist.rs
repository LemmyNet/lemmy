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
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl LocalSiteUrlBlocklist {
  pub async fn replace(pool: &mut DbPool<'_>, url_blocklist: Vec<String>) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;

    conn
      .transaction::<_, Error, _>(|conn| {
        async move {
          Self::clear(conn)
            .await
            .map_err(|_e| diesel::result::Error::NotFound)?;

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
      .with_lemmy_type(LemmyErrorType::CouldntUpdateLocalSiteUrlBlocklist)
  }

  async fn clear(conn: &mut AsyncPgConnection) -> LemmyResult<usize> {
    diesel::delete(local_site_url_blocklist::table)
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::Deleted)
  }

  pub async fn get_all(pool: &mut DbPool<'_>) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    local_site_url_blocklist::table
      .get_results::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}
