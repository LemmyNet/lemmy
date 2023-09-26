use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{DeletePost, PostResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_ban, check_community_deleted_or_removed},
};
use lemmy_db_schema::{
  source::post::{Post, PostUpdateForm},
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyError, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn delete_post(
  data: Json<DeletePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<PostResponse>, LemmyError> {
  let post_id = data.post_id;
  let orig_post = Post::read(&mut context.pool(), post_id).await?;

  // Dont delete it if its already been deleted.
  if orig_post.deleted == data.deleted {
    Err(LemmyErrorType::CouldntUpdatePost)?
  }

  check_community_ban(
    local_user_view.person.id,
    orig_post.community_id,
    &mut context.pool(),
  )
  .await?;
  check_community_deleted_or_removed(orig_post.community_id, &mut context.pool()).await?;

  // Verify that only the creator can delete
  if !Post::is_post_creator(local_user_view.person.id, orig_post.creator_id) {
    Err(LemmyErrorType::NoPostEditAllowed)?
  }

  // Update the post
  let post = Post::update(
    &mut context.pool(),
    data.post_id,
    &PostUpdateForm {
      deleted: Some(data.deleted),
      ..Default::default()
    },
  )
  .await?;

  let person_id = local_user_view.person.id;
  ActivityChannel::submit_activity(
    SendActivityData::DeletePost(post, local_user_view.person, data.0.clone()),
    &context,
  )
  .await?;

  build_post_response(&context, orig_post.community_id, person_id, data.post_id).await
}
