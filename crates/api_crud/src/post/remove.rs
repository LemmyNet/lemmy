use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_post_response,
  context::LemmyContext,
  post::{PostResponse, RemovePost},
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_mod_action,
};
use lemmy_db_schema::{
  source::{
    moderator::{ModRemovePost, ModRemovePostForm},
    post::{Post, PostUpdateForm},
    post_report::PostReport,
  },
  traits::{Crud, Reportable},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn remove_post(
  data: Json<RemovePost>,
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
  let removed = data.removed;
  let post = Post::update(
    &mut context.pool(),
    post_id,
    &PostUpdateForm {
      removed: Some(removed),
      ..Default::default()
    },
  )
  .await?;

  PostReport::resolve_all_for_object(&mut context.pool(), post_id, local_user_view.person.id)
    .await?;

  // Mod tables
  let form = ModRemovePostForm {
    mod_person_id: local_user_view.person.id,
    post_id: data.post_id,
    removed: Some(removed),
    reason: data.reason.clone(),
  };
  ModRemovePost::create(&mut context.pool(), &form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemovePost {
      post,
      moderator: local_user_view.person.clone(),
      reason: data.reason.clone(),
      removed: data.removed,
    },
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
