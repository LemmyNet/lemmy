use crate::structs::SiteView;
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  schema::{local_site, local_site_rate_limit, site},
  utils::{get_conn, DbPool},
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

impl SiteView {
  pub async fn read_local(pool: &mut DbPool<'_>) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      site::table
        .inner_join(local_site::table)
        .inner_join(
          local_site_rate_limit::table.on(local_site::id.eq(local_site_rate_limit::local_site_id)),
        )
        .select(Self::as_select())
        .first(conn)
        .await
        .optional()?
        .ok_or(LemmyErrorType::LocalSiteNotSetup)?,
    )
  }
}
