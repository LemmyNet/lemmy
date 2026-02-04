use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  build_response::build_post_response,
  context::LemmyContext,
  notify::notify_mod_action,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_mod_action, remove_or_restore_post_comments},
};
use lemmy_db_schema::{
  newtypes::PostId,
  source::{
    comment_report::CommentReport,
    community::Community,
    local_user::LocalUser,
    modlog::{Modlog, ModlogInsertForm},
    post::{Post, PostUpdateForm},
    post_report::PostReport,
  },
  traits::Reportable,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::api::{PostResponse, RemovePost, RemovePostWithChildren};
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::LemmyResult;

async fn do_remove(
  context: &Data<LemmyContext>,
  post_id: PostId,
  removed: bool,
  reason: &str,
  local_user_view: &LocalUserView,
) -> LemmyResult<(Post, Community)> {
  // We cannot use PostView to avoid a database read here, as it doesn't return removed items
  // by default. So we would have to pass in `is_mod_or_admin`, but that is impossible without
  // knowing which community the post belongs to.
  let orig_post = Post::read(&mut context.pool(), post_id).await?;
  let community = Community::read(&mut context.pool(), orig_post.community_id).await?;

  check_community_mod_action(local_user_view, &community, false, &mut context.pool()).await?;

  LocalUser::is_higher_mod_or_admin_check(
    &mut context.pool(),
    orig_post.community_id,
    local_user_view.person.id,
    vec![orig_post.creator_id],
  )
  .await?;

  // Update the post
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
  let form = ModlogInsertForm::mod_remove_post(local_user_view.person.id, &post, removed, reason);
  let action = Modlog::create(&mut context.pool(), &[form]).await?;
  notify_mod_action(action, context.app_data());

  Ok((post, community))
}

pub async fn remove_post(
  Json(data): Json<RemovePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let post_id = data.post_id;

  let (post, community) = do_remove(
    &context,
    post_id,
    data.removed,
    &data.reason,
    &local_user_view,
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemovePost {
      post,
      moderator: local_user_view.person.clone(),
      reason: data.reason.clone(),
      removed: data.removed,
      with_replies: None,
    },
    &context,
  )?;

  build_post_response(&context, community.id, local_user_view, post_id).await
}

pub async fn remove_post_with_children(
  Json(data): Json<RemovePostWithChildren>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let post_id = data.post_id;

  let (post, community) = do_remove(
    &context,
    post_id,
    data.removed,
    &data.reason,
    &local_user_view,
  )
  .await?;

  remove_or_restore_post_comments(
    &post,
    local_user_view.person.id,
    data.removed,
    &data.reason,
    &context,
  )
  .await?;

  CommentReport::resolve_all_for_post(&mut context.pool(), post.id, local_user_view.person.id)
    .await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemovePost {
      post,
      moderator: local_user_view.person.clone(),
      reason: data.reason.clone(),
      removed: data.removed,
      with_replies: Some(data.removed),
    },
    &context,
  )?;

  build_post_response(&context, community.id, local_user_view, post_id).await
}
