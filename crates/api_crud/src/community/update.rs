use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{CommunityResponse, EditCommunity},
  utils::{blocking, get_local_user_view_from_jwt},
};
use lemmy_apub::protocol::activities::community::update::UpdateCommunity;
use lemmy_db_schema::{
  newtypes::PersonId,
  source::community::{Community, CommunityForm},
  traits::Crud,
  utils::{diesel_option_overwrite, diesel_option_overwrite_to_url, naive_now},
};
use lemmy_db_views_actor::structs::CommunityModeratorView;
use lemmy_utils::{error::LemmyError, utils::check_slurs_opt, ConnectionId};
use lemmy_websocket::{send::send_community_ws_message, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &EditCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let icon = diesel_option_overwrite_to_url(&data.icon)?;
    let banner = diesel_option_overwrite_to_url(&data.banner)?;
    let description = diesel_option_overwrite(&data.description);

    check_slurs_opt(&data.title, &context.settings().slur_regex())?;
    check_slurs_opt(&data.description, &context.settings().slur_regex())?;

    // Verify its a mod (only mods can edit it)
    let community_id = data.community_id;
    let mods: Vec<PersonId> = blocking(context.pool(), move |conn| {
      CommunityModeratorView::for_community(conn, community_id)
        .map(|v| v.into_iter().map(|m| m.moderator.id).collect())
    })
    .await??;
    if !mods.contains(&local_user_view.person.id) {
      return Err(LemmyError::from_message("not_a_moderator"));
    }

    let community_id = data.community_id;
    let read_community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let community_form = CommunityForm {
      name: read_community.name,
      title: data.title.to_owned().unwrap_or(read_community.title),
      description,
      icon,
      banner,
      nsfw: data.nsfw,
      posting_restricted_to_mods: data.posting_restricted_to_mods,
      updated: Some(naive_now()),
      ..CommunityForm::default()
    };

    let community_id = data.community_id;
    let updated_community = blocking(context.pool(), move |conn| {
      Community::update(conn, community_id, &community_form)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_community"))?;

    UpdateCommunity::send(
      updated_community.into(),
      &local_user_view.person.into(),
      context,
    )
    .await?;

    let op = UserOperationCrud::EditCommunity;
    send_community_ws_message(data.community_id, op, websocket_id, None, context).await
  }
}
