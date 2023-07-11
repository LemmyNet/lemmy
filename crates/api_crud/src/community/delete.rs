use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::build_community_response,
  community::{CommunityResponse, DeleteCommunity},
  context::LemmyContext,
  utils::{is_top_mod, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::community::{Community, CommunityUpdateForm},
  traits::Crud,
};
use lemmy_db_views_actor::structs::CommunityModeratorView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeleteCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CommunityResponse, LemmyError> {
    let data: &DeleteCommunity = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    // Fetch the community mods
    let community_id = data.community_id;
    let community_mods =
      CommunityModeratorView::for_community(&mut context.pool(), community_id).await?;

    // Make sure deleter is the top mod
    is_top_mod(&local_user_view, &community_mods)?;

    // Do the delete
    let community_id = data.community_id;
    let deleted = data.deleted;
    Community::update(
      &mut context.pool(),
      community_id,
      &CommunityUpdateForm::builder()
        .deleted(Some(deleted))
        .build(),
    )
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateCommunity)?;

    build_community_response(context, local_user_view, community_id).await
  }
}
