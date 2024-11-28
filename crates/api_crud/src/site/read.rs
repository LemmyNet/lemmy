use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  site::{GetSiteResponse, MyUserInfo},
};
use lemmy_db_schema::source::{
  actor_language::{LocalUserLanguage, SiteLanguage},
  community_block::CommunityBlock,
  instance_block::InstanceBlock,
  language::Language,
  local_site_url_blocklist::LocalSiteUrlBlocklist,
  oauth_provider::OAuthProvider,
  person_block::PersonBlock,
  tagline::Tagline,
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_db_views_actor::structs::{CommunityFollowerView, CommunityModeratorView, PersonView};
use lemmy_utils::{
  build_cache,
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  CacheLock,
  VERSION,
};
use std::sync::LazyLock;

#[tracing::instrument(skip(context))]
pub async fn get_site(
  local_user_view: Option<LocalUserView>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GetSiteResponse>> {
  // This data is independent from the user account so we can cache it across requests
  static CACHE: CacheLock<GetSiteResponse> = LazyLock::new(build_cache);
  let mut site_response = CACHE
    .try_get_with((), read_site(&context))
    .await
    .map_err(|e| anyhow::anyhow!("Failed to construct site response: {e}"))?;

  // Build the local user with parallel queries and add it to site response
  site_response.my_user = if let Some(ref local_user_view) = local_user_view {
    let person_id = local_user_view.person.id;
    let local_user_id = local_user_view.local_user.id;
    let pool = &mut context.pool();

    let (
      follows,
      community_blocks,
      instance_blocks,
      person_blocks,
      moderates,
      discussion_languages,
    ) = lemmy_db_schema::try_join_with_pool!(pool => (
      |pool| CommunityFollowerView::for_person(pool, person_id),
      |pool| CommunityBlock::for_person(pool, person_id),
      |pool| InstanceBlock::for_person(pool, person_id),
      |pool| PersonBlock::for_person(pool, person_id),
      |pool| CommunityModeratorView::for_person(pool, person_id, Some(&local_user_view.local_user)),
      |pool| LocalUserLanguage::read(pool, local_user_id)
    ))
    .with_lemmy_type(LemmyErrorType::SystemErrLogin)?;

    Some(MyUserInfo {
      local_user_view: local_user_view.clone(),
      follows,
      moderates,
      community_blocks,
      instance_blocks,
      person_blocks,
      discussion_languages,
    })
  } else {
    None
  };

  // filter oauth_providers for public access
  if !local_user_view
    .map(|l| l.local_user.admin)
    .unwrap_or_default()
  {
    site_response.admin_oauth_providers = None;
  }

  Ok(Json(site_response))
}

async fn read_site(context: &LemmyContext) -> LemmyResult<GetSiteResponse> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let admins = PersonView::admins(&mut context.pool()).await?;
  let all_languages = Language::read_all(&mut context.pool()).await?;
  let discussion_languages = SiteLanguage::read_local_raw(&mut context.pool()).await?;
  let blocked_urls = LocalSiteUrlBlocklist::get_all(&mut context.pool()).await?;
  let tagline = Tagline::get_random(&mut context.pool()).await.ok();
  let admin_oauth_providers = OAuthProvider::get_all(&mut context.pool()).await?;
  let oauth_providers = OAuthProvider::convert_providers_to_public(admin_oauth_providers.clone());

  Ok(GetSiteResponse {
    site_view,
    admins,
    version: VERSION.to_string(),
    my_user: None,
    all_languages,
    discussion_languages,
    blocked_urls,
    tagline,
    oauth_providers: Some(oauth_providers),
    admin_oauth_providers: Some(admin_oauth_providers),
    taglines: vec![],
    custom_emojis: vec![],
  })
}
