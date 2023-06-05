use crate::{
  api::PerformApub,
  fetcher::resolve_actor_identifier,
  objects::community::ApubCommunity,
};
use activitypub_federation::config::Data;
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
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait]
impl PerformApub for GetCommunity {
  type Response = GetCommunityResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
  ) -> Result<GetCommunityResponse, LemmyError> {
    let data: &GetCommunity = self;
    let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), context).await;
    let local_site = LocalSite::read(context.pool()).await?;

    if data.name.is_none() && data.id.is_none() {
      return Err(LemmyError::from_message("no_id_given"));
    }

    check_private_instance(&local_user_view, &local_site)?;

    let person_id = local_user_view.as_ref().map(|u| u.person.id);

    let community_id = match data.id {
      Some(id) => id,
      None => {
        let name = data.name.clone().unwrap_or_else(|| "main".to_string());
        resolve_actor_identifier::<ApubCommunity, Community>(&name, context, &local_user_view, true)
          .await
          .map_err(|e| e.with_message("couldnt_find_community"))?
          .id
      }
    };

    let is_mod_or_admin =
      is_mod_or_admin_opt(context.pool(), local_user_view.as_ref(), Some(community_id))
        .await
        .is_ok();

    let community_view = CommunityView::read(
      context.pool(),
      community_id,
      person_id,
      Some(is_mod_or_admin),
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_community"))?;

    let moderators = CommunityModeratorView::for_community(context.pool(), community_id)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_community"))?;

    let site_id =
      Site::instance_actor_id_from_url(community_view.community.actor_id.clone().into());
    let mut site = Site::read_from_apub_id(context.pool(), &site_id.into()).await?;
    // no need to include metadata for local site (its already available through other endpoints).
    // this also prevents us from leaking the federation private key.
    if let Some(s) = &site {
      if s.actor_id.domain() == Some(context.settings().hostname.as_ref()) {
        site = None;
      }
    }

    let community_id = community_view.community.id;
    let discussion_languages = CommunityLanguage::read(context.pool(), community_id).await?;

    let res = GetCommunityResponse {
      community_view,
      site,
      moderators,
      discussion_languages,
    };

    // Return the jwt
    Ok(res)
  }
}
