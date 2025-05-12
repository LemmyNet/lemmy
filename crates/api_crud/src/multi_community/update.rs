use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  community::UpdateMultiCommunity,
  context::LemmyContext,
  LemmyErrorType,
  SuccessResponse,
};
use lemmy_db_schema::source::multi_community::MultiCommunity;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn update_multi_community(
  data: Json<UpdateMultiCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // check that owner is correct
  let read = MultiCommunity::read(&mut context.pool(), data.id).await?;
  if read.owner_id != local_user_view.person.id {
    return Err(LemmyErrorType::NotFound.into());
  }
  // TODO: disallow removed/deleted communities
  MultiCommunity::update(&mut context.pool(), data.id, data.communities.clone()).await?;
  Ok(Json(SuccessResponse::default()))
}
