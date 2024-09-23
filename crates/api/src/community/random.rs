use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  community::CommunityResponse,
  context::LemmyContext,
  utils::{check_private_instance, is_mod_or_admin_opt},
};
use lemmy_db_schema::source::{
  actor_language::CommunityLanguage,
  community::Community,
  local_site::LocalSite,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn get_random_community(
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<CommunityResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site)?;

  let local_user = local_user_view.as_ref().map(|u| &u.local_user);

  let random_community_id = Community::get_random_local_community(&mut context.pool())
    .await?
    .ok_or(LemmyErrorType::NotFound)?
    .id;

  let is_mod_or_admin = is_mod_or_admin_opt(
    &mut context.pool(),
    local_user_view.as_ref(),
    Some(random_community_id),
  )
  .await
  .is_ok();

  let community_view = CommunityView::read(
    &mut context.pool(),
    random_community_id,
    local_user,
    is_mod_or_admin,
  )
  .await?;

  let discussion_languages =
    CommunityLanguage::read(&mut context.pool(), random_community_id).await?;

  Ok(Json(CommunityResponse {
    community_view,
    discussion_languages,
  }))
}
