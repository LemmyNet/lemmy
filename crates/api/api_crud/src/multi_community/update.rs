use super::{check_multi_community_creator, send_federation_update};
use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  source::multi_community::{MultiCommunity, MultiCommunityUpdateForm},
  traits::Crud,
  utils::diesel_string_update,
};
use lemmy_db_views_community::api::UpdateMultiCommunity;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn update_multi_community(
  data: Json<UpdateMultiCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  check_multi_community_creator(data.id, &local_user_view, &context).await?;

  let form = MultiCommunityUpdateForm {
    title: diesel_string_update(data.title.as_deref()),
    description: diesel_string_update(data.description.as_deref()),
    deleted: data.deleted,
    updated_at: Some(Utc::now()),
  };
  let multi = MultiCommunity::update(&mut context.pool(), data.id, &form).await?;

  send_federation_update(multi, local_user_view, &context).await?;

  Ok(Json(SuccessResponse::default()))
}
