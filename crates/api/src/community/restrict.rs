use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  community::{CommunityResponse, RestrictCommunity},
  get_local_user_view_from_jwt,
  is_mod_or_admin,
};
use lemmy_apub::protocol::activities::community::update::UpdateCommunity;
use lemmy_db_schema::{
  naive_now,
  source::community::{Community, CommunityForm},
  traits::Crud,
};
use lemmy_utils::{ConnectionId, LemmyError};
use lemmy_websocket::{send::send_community_ws_message, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl Perform for RestrictCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &RestrictCommunity = self;

    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;
    is_mod_or_admin(context.pool(), local_user_view.person.id, data.community_id).await?;

    let community_id = data.community_id;
    let read_community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;

    let community_form = CommunityForm {
      name: read_community.name,
      title: read_community.title,
      posting_restricted: Some(data.restricted),
      updated: Some(naive_now()),
      ..CommunityForm::default()
    };

    let community_id = data.community_id;
    let updated_community = blocking(context.pool(), move |conn| {
      Community::update(conn, community_id, &community_form)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_community_hidden_status"))?;

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
