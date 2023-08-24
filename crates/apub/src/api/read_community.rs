use crate::{fetcher::resolve_actor_identifier, objects::community::ApubCommunity};
use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_common::{
  community::{GetCommunity, GetCommunityResponse},
  context::LemmyContext,
  utils::{check_private_instance, is_mod_or_admin_opt, local_user_view_from_jwt_opt},
};
use lemmy_db_schema::source::{
  actor_language::CommunityLanguage,
  community::Community,
  local_site::LocalSite,
  site::Site,
};
use lemmy_db_views_actor::structs::{CommunityModeratorView, CommunityView};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorExt2, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn get_community(
  data: Query<GetCommunity>,
  context: Data<LemmyContext>,
) -> Result<Json<GetCommunityResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), &context).await;
  let local_site = LocalSite::read(&mut context.pool()).await?;

  if data.name.is_none() && data.id.is_none() {
    Err(LemmyErrorType::NoIdGiven)?
  }

  check_private_instance(&local_user_view, &local_site)?;

  let person_id = local_user_view.as_ref().map(|u| u.person.id);

  let community_id = match data.id {
    Some(id) => id,
    None => {
      let name = data.name.clone().unwrap_or_else(|| "main".to_string());
      resolve_actor_identifier::<ApubCommunity, Community>(&name, &context, &local_user_view, true)
        .await
        .with_lemmy_type(LemmyErrorType::CouldntFindCommunity)?
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
    person_id,
    is_mod_or_admin,
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntFindCommunity)?;

  let moderators = CommunityModeratorView::for_community(&mut context.pool(), community_id)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntFindCommunity)?;

  let site_id = Site::instance_actor_id_from_url(community_view.community.actor_id.clone().into());
  let mut site = Site::read_from_apub_id(&mut context.pool(), &site_id.into()).await?;
  // no need to include metadata for local site (its already available through other endpoints).
  // this also prevents us from leaking the federation private key.
  if let Some(s) = &site {
    if s.actor_id.domain() == Some(context.settings().hostname.as_ref()) {
      site = None;
    }
  }

  let community_id = community_view.community.id;
  let discussion_languages = CommunityLanguage::read(&mut context.pool(), community_id).await?;

  Ok(Json(GetCommunityResponse {
    community_view,
    site,
    moderators,
    discussion_languages,
  }))
}
