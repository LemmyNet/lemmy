use crate::{context::LemmyContext, utils::is_mod_or_admin};
use actix_web::web::Json;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, PostId},
  source::actor_language::CommunityLanguage,
};
use lemmy_db_schema_file::InstanceId;
use lemmy_db_views_comment::{CommentView, api::CommentResponse};
use lemmy_db_views_community::{CommunityView, api::CommunityResponse};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{PostView, api::PostResponse};
use lemmy_utils::error::LemmyResult;

pub async fn build_comment_response(
  context: &LemmyContext,
  comment_id: CommentId,
  local_user_view: Option<LocalUserView>,
  local_instance_id: InstanceId,
) -> LemmyResult<CommentResponse> {
  let local_user = local_user_view.map(|l| l.local_user);
  let comment_view = CommentView::read(
    &mut context.pool(),
    comment_id,
    local_user.as_ref(),
    local_instance_id,
  )
  .await?;
  Ok(CommentResponse { comment_view })
}

pub async fn build_community_response(
  context: &LemmyContext,
  local_user_view: LocalUserView,
  community_id: CommunityId,
) -> LemmyResult<Json<CommunityResponse>> {
  let is_mod_or_admin = is_mod_or_admin(&mut context.pool(), &local_user_view, community_id)
    .await
    .is_ok();
  let local_user = local_user_view.local_user;
  let community_view = CommunityView::read(
    &mut context.pool(),
    community_id,
    Some(&local_user),
    is_mod_or_admin,
  )
  .await?;
  let discussion_languages = CommunityLanguage::read(&mut context.pool(), community_id).await?;

  Ok(Json(CommunityResponse {
    community_view,
    discussion_languages,
  }))
}

pub async fn build_post_response(
  context: &LemmyContext,
  community_id: CommunityId,
  local_user_view: LocalUserView,
  post_id: PostId,
) -> LemmyResult<Json<PostResponse>> {
  let is_mod_or_admin = is_mod_or_admin(&mut context.pool(), &local_user_view, community_id)
    .await
    .is_ok();
  let local_user = local_user_view.local_user;
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user),
    local_user_view.person.instance_id,
    is_mod_or_admin,
  )
  .await?;
  Ok(Json(PostResponse { post_view }))
}
