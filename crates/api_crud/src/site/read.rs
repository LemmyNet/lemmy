use actix_web::web::{Data, Json};
use futures::try_join;
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

#[tracing::instrument(skip(context))]
pub async fn get_site(
  local_user_view: Option<LocalUserView>,
  context: Data<LemmyContext>,
) -> Result<Json<GetSiteResponse>, LemmyError> {
  // Build the local user
  let my_user = if let Some(local_user_view) = local_user_view {
    let person_id = local_user_view.person.id;
    let local_user_id = local_user_view.local_user.id;

    let (
      follows,
      community_blocks,
      instance_blocks,
      person_blocks,
      moderates,
      discussion_languages,
    ) = try_join!(
      CommunityFollowerView::for_person(context.inner_pool(), person_id),
      CommunityBlockView::for_person(context.inner_pool(), person_id),
      InstanceBlockView::for_person(context.inner_pool(), person_id),
      PersonBlockView::for_person(context.inner_pool(), person_id),
      CommunityModeratorView::for_person(context.inner_pool(), person_id),
      LocalUserLanguage::read(context.inner_pool(), local_user_id)
    )
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

  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let admins = PersonView::admins(&mut context.pool()).await?;
  let all_languages = Language::read_all(&mut context.pool()).await?;
  let discussion_languages = SiteLanguage::read_local_raw(&mut context.pool()).await?;
  let taglines = Tagline::get_all(&mut context.pool(), site_view.local_site.id).await?;
  let custom_emojis =
    CustomEmojiView::get_all(&mut context.pool(), site_view.local_site.id).await?;

  Ok(Json(GetSiteResponse {
    site_view,
    admins,
    version: version::VERSION.to_string(),
    my_user,
    all_languages,
    discussion_languages,
    taglines,
    custom_emojis,
  }))
}
