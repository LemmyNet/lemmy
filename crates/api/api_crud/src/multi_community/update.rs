use super::{check_multi_community_creator, send_federation_update};
use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::{
  source::multi_community::{MultiCommunity, MultiCommunityUpdateForm},
  traits::Crud,
  utils::diesel_string_update,
};
use lemmy_db_views_community::{
  api::{MultiCommunityResponse, UpdateMultiCommunity},
  MultiCommunityView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn update_multi_community(
  data: Json<UpdateMultiCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<MultiCommunityResponse>> {
  let multi_community_id = data.id;
  let my_person_id = local_user_view.person.id;
  check_local_user_valid(&local_user_view)?;

  let orig_multi = MultiCommunity::read(&mut context.pool(), data.id).await?;
  check_multi_community_creator(&orig_multi, &local_user_view)?;

  let form = MultiCommunityUpdateForm {
    title: diesel_string_update(data.title.as_deref()),
    description: diesel_string_update(data.description.as_deref()),
    deleted: data.deleted,
    updated_at: Some(Utc::now()),
  };
  let multi = MultiCommunity::update(&mut context.pool(), multi_community_id, &form).await?;

  send_federation_update(multi, local_user_view.person, &context)?;

  let multi_community_view =
    MultiCommunityView::read(&mut context.pool(), multi_community_id, Some(my_person_id)).await?;

  Ok(Json(MultiCommunityResponse {
    multi_community_view,
  }))
}
