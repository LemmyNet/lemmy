use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_community::{
  MultiCommunityView,
  api::{ListMultiCommunities, ListMultiCommunitiesResponse},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn list_multi_communities(
  data: Query<ListMultiCommunities>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<ListMultiCommunitiesResponse>> {
  let my_person_id = local_user_view.map(|l| l.person.id);
  let multi_communities = MultiCommunityView::list(
    &mut context.pool(),
    data.creator_id,
    my_person_id,
    data.followed_only.unwrap_or_default(),
  )
  .await?;
  Ok(Json(ListMultiCommunitiesResponse { multi_communities }))
}
