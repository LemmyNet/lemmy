use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_community_response,
  community::{CommunityResponse, DeleteCommunity},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_mod_action, is_top_mod},
};
use lemmy_db_schema::{
  source::community::{Community, CommunityUpdateForm},
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::CommunityModeratorView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn delete_community(
  data: Json<DeleteCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<CommunityResponse>, LemmyError> {
  // Fetch the community mods
  let community_id = data.community_id;
  let community_mods =
    CommunityModeratorView::for_community(&mut context.pool(), community_id).await?;

  check_community_mod_action(&local_user_view.person, community_id, &mut context.pool()).await?;

  // Make sure deleter is the top mod
  is_top_mod(&local_user_view, &community_mods)?;

  // Do the delete
  let community_id = data.community_id;
  let deleted = data.deleted;
  let community = Community::update(
    &mut context.pool(),
    community_id,
    &CommunityUpdateForm {
      deleted: Some(deleted),
      ..Default::default()
    },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdateCommunity)?;

  ActivityChannel::submit_activity(
    SendActivityData::DeleteCommunity(local_user_view.person.clone(), community, data.deleted),
    &context,
  )
  .await?;

  build_community_response(&context, local_user_view, community_id).await
}
