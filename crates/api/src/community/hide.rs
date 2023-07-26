use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::build_community_response,
  community::{CommunityResponse, HideCommunity},
  context::LemmyContext,
  utils::{is_admin, local_user_view_from_jwt, sanitize_html_opt},
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityUpdateForm},
    moderator::{ModHideCommunity, ModHideCommunityForm},
  },
  traits::Crud,
};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for HideCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CommunityResponse, LemmyError> {
    let data: &HideCommunity = self;

    // Verify its a admin (only admin can hide or unhide it)
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;
    is_admin(&local_user_view)?;

    let community_form = CommunityUpdateForm::builder()
      .hidden(Some(data.hidden))
      .build();

    let mod_hide_community_form = ModHideCommunityForm {
      community_id: data.community_id,
      mod_person_id: local_user_view.person.id,
      reason: sanitize_html_opt(&data.reason),
      hidden: Some(data.hidden),
    };

    let community_id = data.community_id;
    Community::update(context.pool(), community_id, &community_form)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_community_hidden_status"))?;

    ModHideCommunity::create(context.pool(), &mod_hide_community_form).await?;

    build_community_response(context, local_user_view, community_id).await
  }
}
