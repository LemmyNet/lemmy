use crate::SiteView;
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  source::{
    instance::Instance,
    local_site::{LocalSite, LocalSiteInsertForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
    site::{Site, SiteInsertForm},
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::{instance, local_site, local_site_rate_limit, site};
use lemmy_utils::{
  build_cache,
  error::{LemmyError, LemmyErrorType, LemmyResult},
  CacheLock,
};
use std::sync::{Arc, LazyLock};

impl SiteView {
  pub async fn read_local(pool: &mut DbPool<'_>) -> LemmyResult<Self> {
    static CACHE: CacheLock<SiteView> = LazyLock::new(build_cache);
    CACHE
      .try_get_with((), async move {
        let conn = &mut get_conn(pool).await?;
        let local_site = site::table
          .inner_join(local_site::table)
          .inner_join(instance::table)
          .inner_join(
            local_site_rate_limit::table
              .on(local_site::id.eq(local_site_rate_limit::local_site_id)),
          )
          .select(Self::as_select())
          .first(conn)
          .await
          .optional()?
          .ok_or(LemmyErrorType::LocalSiteNotSetup)?;
        Ok(local_site)
      })
      .await
      .map_err(|e: Arc<LemmyError>| anyhow::anyhow!("err getting local site: {e:?}").into())
  }
}

pub async fn create_test_instance(pool: &mut DbPool<'_>) -> LemmyResult<Instance> {
  let instance = Instance::read_or_create(pool, "example.com".to_string()).await?;
  let site = Site::create(pool, &SiteInsertForm::new("name".to_string(), instance.id)).await?;
  let local_site = LocalSite::create(pool, &LocalSiteInsertForm::new(site.id)).await?;
  LocalSiteRateLimit::create(pool, &LocalSiteRateLimitInsertForm::new(local_site.id)).await?;
  Ok(instance)
}
