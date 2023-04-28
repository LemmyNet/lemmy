use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{CommunityResponse, RemoveCommunity},
  context::LemmyContext,
  sensitive::Sensitive,
  utils::{is_admin, local_user_view_from_jwt},
  websocket::UserOperationCrud,
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityUpdateForm},
    moderator::{ModRemoveCommunity, ModRemoveCommunityForm},
  },
  traits::Crud,
};
use lemmy_utils::{error::LemmyError, utils::time::naive_from_unix, ConnectionId};

#[async_trait::async_trait(?Send)]
impl PerformCrud for RemoveCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    auth: Option<Sensitive<String>>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &RemoveCommunity = self;
    let local_user_view = local_user_view_from_jwt(auth, context).await?;

    // Verify its an admin (only an admin can remove a community)
    is_admin(&local_user_view)?;

    // Do the remove
    let community_id = data.community_id;
    let removed = data.removed;
    Community::update(
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

    let res = context
      .send_community_ws_message(
        &UserOperationCrud::RemoveCommunity,
        data.community_id,
        websocket_id,
        Some(local_user_view.person.id),
      )
      .await?;

    Ok(res)
  }
}
