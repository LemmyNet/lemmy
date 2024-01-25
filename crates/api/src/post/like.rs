use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{CreatePostLike, PostResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_user_action, check_vote_permission, mark_post_as_read},
};
use lemmy_db_schema::{
  source::{
    community::Community,
    local_site::LocalSite,
    post::{Post, PostLike, PostLikeForm},
  },
  traits::{Crud, Likeable},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};
use std::ops::Deref;

#[tracing::instrument(skip(context))]
pub async fn like_post(
  data: Json<CreatePostLike>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<PostResponse>, LemmyError> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  // Check for a community ban
  let post_id = data.post_id;
  let post = Post::read(&mut context.pool(), post_id).await?;

  check_vote_permission(data.score, &local_site, &local_user_view.person).await?;

  check_community_user_action(
    &local_user_view.person,
    post.community_id,
    &mut context.pool(),
  )
  .await?;

  let like_form = PostLikeForm {
    post_id: data.post_id,
    person_id: local_user_view.person.id,
    score: data.score,
  };

  // Remove any likes first
  let person_id = local_user_view.person.id;

  PostLike::remove(&mut context.pool(), person_id, post_id).await?;

  // Only add the like if the score isnt 0
  let do_add = like_form.score != 0 && (like_form.score == 1 || like_form.score == -1);
  if do_add {
    PostLike::like(&mut context.pool(), &like_form)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntLikePost)?;
  }

  // Mark the post as read
  mark_post_as_read(person_id, post_id, &mut context.pool()).await?;

  ActivityChannel::submit_activity(
    SendActivityData::LikePostOrComment(
      post.ap_id,
      local_user_view.person.clone(),
      Community::read(&mut context.pool(), post.community_id).await?,
      data.score,
    ),
    &context,
  )
  .await?;

  build_post_response(
    context.deref(),
    post.community_id,
    &local_user_view.person,
    post_id,
  )
  .await
}
