use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{LockPost, PostResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_mod_action,
};
use lemmy_db_schema::{
  source::{
    moderator::{ModLockPost, ModLockPostForm},
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn lock_post(
  data: Json<LockPost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let post_id = data.post_id;
  let orig_post = Post::read(&mut context.pool(), post_id).await?;

  check_community_mod_action(
    &local_user_view.person,
    orig_post.community_id,
    false,
    &mut context.pool(),
  )
  .await?;

  // Update the post
  let post_id = data.post_id;
  let locked = data.locked;
  let post = Post::update(
    &mut context.pool(),
    post_id,
    &PostUpdateForm {
      locked: Some(locked),
      ..Default::default()
    },
  )
  .await?;

  // Mod tables
  let form = ModLockPostForm {
    mod_person_id: local_user_view.person.id,
    post_id: data.post_id,
    locked: Some(locked),
  };
  ModLockPost::create(&mut context.pool(), &form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::LockPost(post, local_user_view.person.clone(), data.locked),
    &context,
  )
  .await?;

  build_post_response(
    &context,
    orig_post.community_id,
    &local_user_view.person,
    post_id,
  )
  .await
}
