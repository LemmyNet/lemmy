use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  site::{GetSiteResponse, MyUserInfo},
};
use lemmy_db_schema::source::{
  actor_language::{LocalUserLanguage, SiteLanguage},
  language::Language,
  tagline::Tagline,
};
use lemmy_db_views::structs::{CustomEmojiView, LocalUserView, SiteView};
use lemmy_db_views_actor::structs::{
  CommunityBlockView,
  CommunityFollowerView,
  CommunityModeratorView,
  InstanceBlockView,
  PersonBlockView,
  PersonView,
};
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  version,
};
use moka::future::Cache;
use once_cell::sync::Lazy;
use std::time::Duration;

#[tracing::instrument(skip(context))]
pub async fn get_site(
  local_user_view: Option<LocalUserView>,
  context: Data<LemmyContext>,
) -> Result<Json<GetSiteResponse>, LemmyError> {
  static CACHE: Lazy<Cache<(), GetSiteResponse>> = Lazy::new(|| {
    Cache::builder()
      .max_capacity(1)
      .time_to_live(Duration::from_secs(1))
      .build()
  });

  // This data is independent from the user account so we can cache it across requests
  let mut site_response = CACHE
    .try_get_with::<_, LemmyError>((), async {
      let site_view = SiteView::read_local(&mut context.pool()).await?;
      let admins = PersonView::admins(&mut context.pool()).await?;
      let all_languages = Language::read_all(&mut context.pool()).await?;
      let discussion_languages = SiteLanguage::read_local_raw(&mut context.pool()).await?;
      let taglines = Tagline::get_all(&mut context.pool(), site_view.local_site.id).await?;
      let custom_emojis =
        CustomEmojiView::get_all(&mut context.pool(), site_view.local_site.id).await?;
      Ok(GetSiteResponse {
        site_view,
        admins,
        version: version::VERSION.to_string(),
        my_user: None,
        all_languages,
        discussion_languages,
        taglines,
        custom_emojis,
      })
    })
    .await
    .map_err(|e| anyhow::anyhow!("Failed to construct site response: {e}"))?;

  // Build the local user with parallel queries and add it to site response
  site_response.my_user = if let Some(local_user_view) = local_user_view {
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
      |pool| CommunityBlockView::for_person(pool, person_id),
      |pool| InstanceBlockView::for_person(pool, person_id),
      |pool| PersonBlockView::for_person(pool, person_id),
      |pool| CommunityModeratorView::for_person(pool, person_id, true),
      |pool| LocalUserLanguage::read(pool, local_user_id)
    ))
    .with_lemmy_type(LemmyErrorType::SystemErrLogin)?;

    Some(MyUserInfo {
      local_user_view,
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

  Ok(Json(site_response))
}
