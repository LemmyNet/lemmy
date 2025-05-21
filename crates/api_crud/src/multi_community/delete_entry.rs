use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  community::CreateOrDeleteMultiCommunityEntry,
  context::LemmyContext,
  LemmyErrorType,
  SuccessResponse,
};
use lemmy_db_schema::source::multi_community::MultiCommunity;
use lemmy_db_views_community::{multi_community::ReadParams, MultiCommunityView};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn delete_multi_community_entry(
  data: Json<CreateOrDeleteMultiCommunityEntry>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // check that owner is correct
  let read = MultiCommunityView::read(&mut context.pool(), ReadParams::Id(data.id)).await?;
  if read.multi.creator_id != local_user_view.person.id {
    return Err(LemmyErrorType::MultiCommunityUpdateWrongUser.into());
  }

  MultiCommunity::delete_entry(&mut context.pool(), data.id, data.community_id).await?;
  Ok(Json(SuccessResponse::default()))
}
