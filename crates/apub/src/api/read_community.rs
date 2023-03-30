use crate::{
  api::PerformApub,
  fetcher::resolve_actor_identifier,
  objects::community::ApubCommunity,
};
use actix_web::web::Data;
use lemmy_api_common::{
  community::{GetCommunity, GetCommunityResponse},
  context::LemmyContext,
  utils::{check_private_instance, get_local_user_view_from_jwt_opt},
};
use lemmy_db_schema::{
  impls::actor_language::default_post_language,
  source::{
    actor_language::CommunityLanguage,
    community::Community,
    local_site::LocalSite,
    site::Site,
  },
  traits::DeleteableOrRemoveable,
};
use lemmy_db_views_actor::structs::{CommunityModeratorView, CommunityView};
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl PerformApub for GetCommunity {
  type Response = GetCommunityResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetCommunityResponse, LemmyError> {
    let data: &GetCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt_opt(data.auth.as_ref(), context.pool(), context.secret())
        .await?;
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
        resolve_actor_identifier::<ApubCommunity, Community>(&name, context, true)
          .await
          .map_err(|e| e.with_message("couldnt_find_community"))?
          .id
      }
    };

    let mut community_view = CommunityView::read(context.pool(), community_id, person_id)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_community"))?;

    // Blank out deleted or removed info for non-logged in users
    if person_id.is_none() && (community_view.community.deleted || community_view.community.removed)
    {
      community_view.community = community_view.community.blank_out_deleted_or_removed_info();
    }

    let moderators = CommunityModeratorView::for_community(context.pool(), community_id)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_community"))?;

    let online = context
      .chat_server()
      .get_community_users_online(community_id)?;

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
    let default_post_language = if let Some(user) = local_user_view {
      default_post_language(context.pool(), community_id, user.local_user.id).await?
    } else {
      None
    };

    let res = GetCommunityResponse {
      community_view,
      site,
      moderators,
      online,
      discussion_languages,
      default_post_language,
    };

    // Return the jwt
    Ok(res)
  }
}
