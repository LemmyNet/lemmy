use crate::fetcher::resolve_ap_identifier;
use activitypub_federation::config::Data;
use actix_web::{web::Json, HttpRequest};
use lemmy_api_utils::{
  context::LemmyContext,
  request::parse_community_id_or_name_from_request,
  utils::{check_private_instance, is_mod_or_admin_opt, read_site_for_actor},
};
use lemmy_apub_objects::objects::community::ApubCommunity;
use lemmy_db_schema::source::{actor_language::CommunityLanguage, community::Community};
use lemmy_db_views_community::{
  api::{CommunityIdOrName, GetCommunityResponse},
  CommunityView,
};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn get_community(
  req: HttpRequest,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GetCommunityResponse>> {
  let community_id_or_name = parse_community_id_or_name_from_request(&req)?;
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;

  check_private_instance(&local_user_view, &local_site)?;

  let local_user = local_user_view.as_ref().map(|u| &u.local_user);

  let community_id = match community_id_or_name {
    CommunityIdOrName::Id(id) => id,
    CommunityIdOrName::Name(name) => {
      resolve_ap_identifier::<ApubCommunity, Community>(name, &context, &local_user_view, true)
        .await?
        .id
    }
  };

  let is_mod_or_admin = is_mod_or_admin_opt(
    &mut context.pool(),
    local_user_view.as_ref(),
    Some(community_id),
  )
  .await
  .is_ok();

  let community_view = CommunityView::read(
    &mut context.pool(),
    community_id,
    local_user,
    is_mod_or_admin,
  )
  .await?;

  let moderators = CommunityModeratorView::for_community(&mut context.pool(), community_id).await?;

  let site = read_site_for_actor(community_view.community.ap_id.clone(), &context).await?;

  let community_id = community_view.community.id;
  let discussion_languages = CommunityLanguage::read(&mut context.pool(), community_id).await?;

  Ok(Json(GetCommunityResponse {
    community_view,
    site,
    moderators,
    discussion_languages,
  }))
}
