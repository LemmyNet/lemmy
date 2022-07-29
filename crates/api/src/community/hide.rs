use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{CommunityResponse, HideCommunity},
  utils::{blocking, get_local_user_view_from_jwt, is_admin},
};
use lemmy_apub::protocol::activities::community::update::UpdateCommunity;
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityForm},
    moderator::{ModHideCommunity, ModHideCommunityForm},
  },
  traits::Crud,
  utils::naive_now,
};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::{send::send_community_ws_message, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl Perform for HideCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &HideCommunity = self;

    // Verify its a admin (only admin can hide or unhide it)
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;
    is_admin(&local_user_view)?;

    let community_id = data.community_id;
    let read_community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let community_form = CommunityForm {
      name: read_community.name,
      title: read_community.title,
      description: Some(read_community.description.to_owned()),
      hidden: Some(data.hidden),
      updated: Some(naive_now()),
      ..CommunityForm::default()
    };

    let mod_hide_community_form = ModHideCommunityForm {
      community_id: data.community_id,
      mod_person_id: local_user_view.person.id,
      reason: data.reason.clone(),
      hidden: Some(data.hidden),
    };

    let community_id = data.community_id;
    let updated_community = blocking(context.pool(), move |conn| {
      Community::update(conn, community_id, &community_form)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_community_hidden_status"))?;

    blocking(context.pool(), move |conn| {
      ModHideCommunity::create(conn, &mod_hide_community_form)
    })
    .await??;

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
