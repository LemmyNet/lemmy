use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{CommunityResponse, RemoveCommunity},
  utils::{get_local_user_view_from_jwt, is_admin},
};
use lemmy_apub::activities::deletion::{send_apub_delete_in_community, DeletableObjects};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityUpdateForm},
    moderator::{ModRemoveCommunity, ModRemoveCommunityForm},
  },
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, utils::naive_from_unix, ConnectionId};
use lemmy_websocket::{send::send_community_ws_message, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for RemoveCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &RemoveCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Verify its an admin (only an admin can remove a community)
    is_admin(&local_user_view)?;

    // Do the remove
    let community_id = data.community_id;
    let removed = data.removed;
    let updated_community = Community::update(
      context.pool(),
      community_id,
      &CommunityUpdateForm::builder()
        .removed(Some(removed))
        .build(),
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_community"))?;

    // Mod tables
    let expires = data.expires.map(naive_from_unix);
    let form = ModRemoveCommunityForm {
      mod_person_id: local_user_view.person.id,
      community_id: data.community_id,
      removed: Some(removed),
      reason: data.reason.clone(),
      expires,
    };
    ModRemoveCommunity::create(context.pool(), &form).await?;

    let res = send_community_ws_message(
      data.community_id,
      UserOperationCrud::RemoveCommunity,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await?;

    // Apub messages
    let deletable = DeletableObjects::Community(Box::new(updated_community.clone().into()));
    send_apub_delete_in_community(
      local_user_view.person,
      updated_community,
      deletable,
      data.reason.clone().or_else(|| Some(String::new())),
      removed,
      context,
    )
    .await?;
    Ok(res)
  }
}
