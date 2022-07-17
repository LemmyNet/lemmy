use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{GetCommunity, GetCommunityResponse},
  utils::{blocking, check_private_instance, get_local_user_view_from_jwt_opt},
};
use lemmy_apub::{
  fetcher::resolve_actor_identifier,
  objects::{community::ApubCommunity, instance::instance_actor_id_from_url},
};
use lemmy_db_schema::{
  source::{community::Community, site::Site},
  traits::DeleteableOrRemoveable,
};
use lemmy_db_views_actor::structs::{CommunityModeratorView, CommunityView};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::{messages::GetCommunityUsersOnline, LemmyContext};

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetCommunity {
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

    if data.name.is_none() && data.id.is_none() {
      return Err(LemmyError::from_message("no_id_given"));
    }

    check_private_instance(&local_user_view, context.pool()).await?;

    let person_id = local_user_view.map(|u| u.person.id);

    let community_id = match data.id {
      Some(id) => id,
      None => {
        let name = data.name.to_owned().unwrap_or_else(|| "main".to_string());
        resolve_actor_identifier::<ApubCommunity, Community>(&name, context)
          .await
          .map_err(|e| e.with_message("couldnt_find_community"))?
          .id
      }
    };

    let mut community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, person_id)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_community"))?;

    // Blank out deleted or removed info for non-logged in users
    if person_id.is_none() && (community_view.community.deleted || community_view.community.removed)
    {
      community_view.community = community_view.community.blank_out_deleted_or_removed_info();
    }

    let moderators: Vec<CommunityModeratorView> = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_find_community"))?;

    let online = context
      .chat_server()
      .send(GetCommunityUsersOnline { community_id })
      .await
      .unwrap_or(1);

    let site_id = instance_actor_id_from_url(community_view.community.actor_id.clone().into());
    let mut site: Option<Site> = blocking(context.pool(), move |conn| {
      Site::read_from_apub_id(conn, site_id)
    })
    .await??;
    // no need to include metadata for local site (its already available through other endpoints).
    // this also prevents us from leaking the federation private key.
    if let Some(s) = &site {
      if s.actor_id.domain() == Some(context.settings().hostname.as_ref()) {
        site = None;
      }
    }

    let res = GetCommunityResponse {
      community_view,
      site,
      moderators,
      online,
    };

    // Return the jwt
    Ok(res)
  }
}
