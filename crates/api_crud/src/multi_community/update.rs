use activitypub_federation::config::Data;
use actix_web::web::Json;
use futures::future::try_join_all;
use lemmy_api_common::{
  community::UpdateMultiCommunity,
  context::LemmyContext,
  LemmyErrorType,
  SuccessResponse,
};
use lemmy_db_schema::{
  source::{community::Community, multi_community::MultiCommunity},
  traits::Crud,
};
use lemmy_db_schema_file::enums::CommunityVisibility;
use lemmy_db_views_community::{multi_community::ReadParams, MultiCommunityView};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{LemmyError, LemmyResult, MAX_API_PARAM_ELEMENTS};

pub async fn update_multi_community(
  data: Json<UpdateMultiCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // check that owner is correct
  let read = MultiCommunityView::read(&mut context.pool(), ReadParams::Id(data.id)).await?;
  if read.multi.owner_id != local_user_view.person.id {
    return Err(LemmyErrorType::NotFound.into());
  }
  if data.communities.len() > MAX_API_PARAM_ELEMENTS {
    Err(LemmyErrorType::TooManyItems)?;
  }

  // Disallow removed/deleted communities
  try_join_all(data.communities.iter().map(|id| async {
    let c = Community::read(&mut context.pool(), *id).await?;
    if c.removed || c.deleted || c.visibility != CommunityVisibility::Public {
      return Err(LemmyErrorType::NotFound.into());
    }
    Ok::<_, LemmyError>(c)
  }))
  .await?;
  MultiCommunity::update(&mut context.pool(), data.id, &data.communities).await?;
  Ok(Json(SuccessResponse::default()))
}
