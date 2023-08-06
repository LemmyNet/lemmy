use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{PostResponse, RemovePost},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_ban, is_mod_or_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{
    moderator::{ModRemovePost, ModRemovePostForm},
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn remove_post(
  data: Json<RemovePost>,
  context: Data<LemmyContext>,
) -> Result<Json<PostResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

  let post_id = data.post_id;
  let orig_post = Post::read(&mut context.pool(), post_id).await?;

  check_community_ban(
    local_user_view.person.id,
    orig_post.community_id,
    &mut context.pool(),
  )
  .await?;

  // Verify that only the mods can remove
  is_mod_or_admin(
    &mut context.pool(),
    local_user_view.person.id,
    orig_post.community_id,
  )
  .await?;

  // Update the post
  let post_id = data.post_id;
  let removed = data.removed;
  let post = Post::update(
    &mut context.pool(),
    post_id,
    &PostUpdateForm::builder().removed(Some(removed)).build(),
  )
  .await?;

  // Mod tables
  let form = ModRemovePostForm {
    mod_person_id: local_user_view.person.id,
    post_id: data.post_id,
    removed,
    reason: data.reason.clone(),
  };
  ModRemovePost::create(&mut context.pool(), &form).await?;

  let person_id = local_user_view.person.id;
  ActivityChannel::submit_activity(
    SendActivityData::RemovePost(post, local_user_view.person, data.0),
    &context,
  )
  .await?;

  build_post_response(&context, orig_post.community_id, person_id, post_id).await
}
