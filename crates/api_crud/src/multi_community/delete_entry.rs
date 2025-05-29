use super::{check_multi_community_creator, send_federation_update};
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  community::CreateOrDeleteMultiCommunityEntry,
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  SuccessResponse,
};
use lemmy_db_schema::{
  source::{community::Community, multi_community::MultiCommunity},
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn delete_multi_community_entry(
  data: Json<CreateOrDeleteMultiCommunityEntry>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let multi = check_multi_community_creator(data.id, &local_user_view, &context).await?;
  let community = Community::read(&mut context.pool(), data.community_id).await?;

  MultiCommunity::delete_entry(&mut context.pool(), data.id, &community).await?;

  send_federation_update(multi, local_user_view, &context).await?;

  Ok(Json(SuccessResponse::default()))
}
