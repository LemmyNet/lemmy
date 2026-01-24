use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, plugins::plugin_metadata};
use lemmy_db_schema::source::{
  actor_language::SiteLanguage,
  language::Language,
  local_site_url_blocklist::LocalSiteUrlBlocklist,
  oauth_provider::AdminOAuthProvider,
  registration_application::RegistrationApplication,
  tagline::Tagline,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::PersonView;
use lemmy_db_views_site::{SiteView, api::GetSiteResponse};
use lemmy_utils::{CacheLock, VERSION, build_cache, error::LemmyResult};
use std::sync::LazyLock;

pub async fn get_site(
  local_user_view: Option<LocalUserView>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetSiteResponse>> {
  // This data is independent from the user account so we can cache it across requests
  static CACHE: CacheLock<GetSiteResponse> = LazyLock::new(build_cache);
  let mut site_response = Box::pin(CACHE.try_get_with((), read_site(&context)))
    .await
    .map_err(|e| anyhow::anyhow!("Failed to construct site response: {e}"))?;

  // filter oauth_providers for public access
  if !local_user_view
    .map(|l| l.local_user.admin)
    .unwrap_or_default()
  {
    site_response.admin_oauth_providers = vec![];
  }

  Ok(Json(site_response))
}

async fn read_site(context: &LemmyContext) -> LemmyResult<GetSiteResponse> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let admins = PersonView::list_admins(None, site_view.instance.id, &mut context.pool()).await?;
  let all_languages = Language::read_all(&mut context.pool()).await?;
  let discussion_languages = SiteLanguage::read_local_raw(&mut context.pool()).await?;
  let blocked_urls = LocalSiteUrlBlocklist::get_all(&mut context.pool()).await?;
  let tagline = Tagline::get_random(&mut context.pool()).await.ok();
  let admin_oauth_providers = AdminOAuthProvider::get_all(&mut context.pool()).await?;
  let oauth_providers =
    AdminOAuthProvider::convert_providers_to_public(admin_oauth_providers.clone());
  let last_application_duration_seconds =
    RegistrationApplication::last_updated(&mut context.pool())
      .await
      .ok()
      .and_then(|u| u.updated_published_duration());

  Ok(GetSiteResponse {
    site_view,
    admins,
    version: VERSION.to_string(),
    all_languages,
    discussion_languages,
    blocked_urls,
    tagline,
    oauth_providers,
    admin_oauth_providers,
    image_upload_disabled: context.settings().pictrs()?.image_upload_disabled,
    active_plugins: plugin_metadata(),
    last_application_duration_seconds,
  })
}
