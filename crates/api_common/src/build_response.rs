use crate::{
  comment::CommentResponse,
  community::CommunityResponse,
  context::LemmyContext,
  post::PostResponse,
  utils::is_mod_or_admin,
};
use actix_web::web::Data;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, LocalUserId, PersonId, PostId},
  source::actor_language::CommunityLanguage,
};
use lemmy_db_views::structs::{CommentView, LocalUserView, PostView};
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::error::LemmyError;

pub async fn build_comment_response(
  context: &Data<LemmyContext>,
  comment_id: CommentId,
  local_user_view: Option<LocalUserView>,
  form_id: Option<String>,
  recipient_ids: Vec<LocalUserId>,
) -> Result<CommentResponse, LemmyError> {
  let person_id = local_user_view.map(|l| l.person.id);
  let comment_view = CommentView::read(context.pool(), comment_id, person_id).await?;
  Ok(CommentResponse {
    comment_view,
    recipient_ids,
    form_id,
  })
}

pub async fn build_community_response(
  context: &Data<LemmyContext>,
  local_user_view: LocalUserView,
  community_id: CommunityId,
) -> Result<CommunityResponse, LemmyError> {
  let is_mod_or_admin = is_mod_or_admin(context.pool(), local_user_view.person.id, community_id)
    .await
    .is_ok();
  let person_id = local_user_view.person.id;
  let community_view = CommunityView::read(
    context.pool(),
    community_id,
    Some(person_id),
    Some(is_mod_or_admin),
  )
  .await?;
  let discussion_languages = CommunityLanguage::read(context.pool(), community_id).await?;

  Ok(CommunityResponse {
    community_view,
    discussion_languages,
  })
}

pub async fn build_post_response(
  context: &Data<LemmyContext>,
  community_id: CommunityId,
  person_id: PersonId,
  post_id: PostId,
) -> Result<PostResponse, LemmyError> {
  let is_mod_or_admin = is_mod_or_admin(context.pool(), person_id, community_id)
    .await
    .is_ok();
  let post_view = PostView::read(
    context.pool(),
    post_id,
    Some(person_id),
    Some(is_mod_or_admin),
  )
  .await?;
  Ok(PostResponse { post_view })
}
