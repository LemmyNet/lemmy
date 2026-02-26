use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  build_response::build_post_response,
  context::LemmyContext,
  notify::notify_mod_action,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_mod_action,
};
use lemmy_db_schema::{
  source::{
    comment::Comment,
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
use lemmy_db_views_post::api::{PostResponse, RemovePost};
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::LemmyResult;

pub async fn remove_post(
  Json(data): Json<RemovePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let post_id = data.post_id;
  let remove_post = data.remove_children.unwrap_or(data.removed);

  // We cannot use PostView to avoid a database read here, as it doesn't return removed items
  // by default. So we would have to pass in `is_mod_or_admin`, but that is impossible without
  // knowing which community the post belongs to.
  let orig_post = Post::read(&mut context.pool(), post_id).await?;
  let community = Community::read(&mut context.pool(), orig_post.community_id).await?;

  check_community_mod_action(&local_user_view, &community, false, &mut context.pool()).await?;

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
      removed: Some(remove_post),
      ..Default::default()
    },
  )
  .await?;

  PostReport::resolve_all_for_object(&mut context.pool(), post_id, local_user_view.person.id)
    .await?;

  // Mod tables
  let form =
    ModlogInsertForm::mod_remove_post(local_user_view.person.id, &post, remove_post, &data.reason);
  let action = Modlog::create(&mut context.pool(), &[form]).await?;
  notify_mod_action(action, context.app_data());

  if let Some(remove_children) = data.remove_children {
    let updated_comments: Vec<Comment> =
      Comment::update_removed_for_post(&mut context.pool(), post_id, remove_children).await?;

    let forms: Vec<_> = updated_comments
      .iter()
      // Filter out deleted comments here so their content doesn't show up in the modlog.
      .filter(|c| !c.deleted)
      .map(|comment| {
        ModlogInsertForm::mod_remove_comment(
          local_user_view.person.id,
          comment,
          community.id,
          remove_children,
          &data.reason,
        )
      })
      .collect();

    let actions = Modlog::create(&mut context.pool(), &forms).await?;
    notify_mod_action(actions, &context);

    CommentReport::resolve_all_for_post(&mut context.pool(), post.id, local_user_view.person.id)
      .await?;
  }

  ActivityChannel::submit_activity(
    SendActivityData::RemovePost {
      post,
      moderator: local_user_view.person.clone(),
      reason: data.reason.clone(),
      removed: remove_post,
      with_replies: data.remove_children.unwrap_or_default(),
    },
    &context,
  )?;

  build_post_response(&context, community.id, local_user_view, post_id).await
}
