use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{CommunityResponse, HideCommunity},
  context::LemmyContext,
  utils::{get_local_user_view_from_jwt, is_admin},
  websocket::UserOperationCrud,
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityUpdateForm},
    moderator::{ModHideCommunity, ModHideCommunityForm},
  },
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, ConnectionId};

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

    let community_form = CommunityUpdateForm::builder()
      .hidden(Some(data.hidden))
      .build();

    let mod_hide_community_form = ModHideCommunityForm {
      community_id: data.community_id,
      mod_person_id: local_user_view.person.id,
      reason: data.reason.clone(),
      hidden: Some(data.hidden),
    };

    let community_id = data.community_id;
    Community::update(context.pool(), community_id, &community_form)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_community_hidden_status"))?;

    ModHideCommunity::create(context.pool(), &mod_hide_community_form).await?;

    context
      .send_community_ws_message(
        &UserOperationCrud::EditCommunity,
        data.community_id,
        websocket_id,
        None,
      )
      .await
  }
}
