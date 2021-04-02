use crate::{community::send_community_websocket, PerformCrud};
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  community::{CommunityResponse, EditCommunity},
  get_local_user_view_from_jwt,
};
use lemmy_db_queries::{diesel_option_overwrite_to_url, Crud};
use lemmy_db_schema::{
  naive_now,
  source::community::{Community, CommunityForm},
  PersonId,
};
use lemmy_db_views_actor::{
  community_moderator_view::CommunityModeratorView,
  community_view::CommunityView,
};
use lemmy_utils::{
  utils::{check_slurs, check_slurs_opt},
  ApiError,
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::{LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditCommunity {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &EditCommunity = &self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    check_slurs(&data.title)?;
    check_slurs_opt(&data.description)?;

    // Verify its a mod (only mods can edit it)
    let community_id = data.community_id;
    let mods: Vec<PersonId> = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
        .map(|v| v.into_iter().map(|m| m.moderator.id).collect())
    })
    .await??;
    if !mods.contains(&local_user_view.person.id) {
      return Err(ApiError::err("not_a_moderator").into());
    }

    let community_id = data.community_id;
    let read_community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let icon = diesel_option_overwrite_to_url(&data.icon)?;
    let banner = diesel_option_overwrite_to_url(&data.banner)?;

    let community_form = CommunityForm {
      name: read_community.name,
      title: data.title.to_owned(),
      description: data.description.to_owned(),
      icon,
      banner,
      nsfw: data.nsfw,
      updated: Some(naive_now()),
      ..CommunityForm::default()
    };

    let community_id = data.community_id;
    match blocking(context.pool(), move |conn| {
      Community::update(conn, community_id, &community_form)
    })
    .await?
    {
      Ok(community) => community,
      Err(_e) => return Err(ApiError::err("couldnt_update_community").into()),
    };

    // TODO there needs to be some kind of an apub update
    // process for communities and users

    let community_id = data.community_id;
    let person_id = local_user_view.person.id;
    let community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, community_id, Some(person_id))
    })
    .await??;

    let res = CommunityResponse { community_view };

    send_community_websocket(
      &res,
      context,
      websocket_id,
      UserOperationCrud::EditCommunity,
    );

    Ok(res)
  }
}
