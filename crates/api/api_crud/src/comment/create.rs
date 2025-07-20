use crate::community_use_pending;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  build_response::build_comment_response,
  context::LemmyContext,
  notify::NotifyData,
  plugins::{plugin_hook_after, plugin_hook_before},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_comment_depth,
    check_community_user_action,
    check_post_deleted_or_removed,
    get_url_blocklist,
    is_mod_or_admin,
    process_markdown,
    slur_regex,
    update_read_comments,
  },
};
use lemmy_db_schema::{
  impls::actor_language::validate_post_language,
  source::{
    comment::{Comment, CommentActions, CommentInsertForm, CommentLikeForm},
    notification::Notification,
  },
  traits::{Crud, Likeable},
};
use lemmy_db_views_comment::api::{CommentResponse, CreateComment};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::PostView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  utils::validation::is_valid_body_field,
};

pub async fn create_comment(
  data: Json<CreateComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponse>> {
  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown(&data.content, &slur_regex, &url_blocklist, &context).await?;
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;
  is_valid_body_field(&content, false)?;

  // Check for a community ban
  let post_id = data.post_id;

  let local_instance_id = local_user_view.person.instance_id;

  // Read the full post view in order to get the comments count.
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    local_instance_id,
    true,
  )
  .await?;

  let post = post_view.post;
  let community_id = post_view.community.id;

  check_community_user_action(&local_user_view, &post_view.community, &mut context.pool()).await?;
  check_post_deleted_or_removed(&post)?;

  // Check if post is locked, no new comments
  let is_mod_or_admin = is_mod_or_admin(&mut context.pool(), &local_user_view, community_id)
    .await
    .is_ok();
  if post.locked && !is_mod_or_admin {
    Err(LemmyErrorType::Locked)?
  }

  // Fetch the parent, if it exists
  let parent_opt = if let Some(parent_id) = data.parent_id {
    Comment::read(&mut context.pool(), parent_id).await.ok()
  } else {
    None
  };

  // If there's a parent_id, check to make sure that comment is in that post
  // Strange issue where sometimes the post ID of the parent comment is incorrect
  if let Some(parent) = parent_opt.as_ref() {
    if parent.post_id != post_id {
      Err(LemmyErrorType::CouldntCreateComment)?
    }
    check_comment_depth(parent)?;
  }

  let language_id = validate_post_language(
    &mut context.pool(),
    data.language_id,
    community_id,
    local_user_view.local_user.id,
  )
  .await?;

  let mut comment_form = CommentInsertForm {
    language_id: Some(language_id),
    federation_pending: Some(community_use_pending(&post_view.community, &context).await),
    ..CommentInsertForm::new(local_user_view.person.id, data.post_id, content.clone())
  };
  comment_form = plugin_hook_before("before_create_local_comment", comment_form).await?;

  // Create the comment
  let parent_path = parent_opt.clone().map(|t| t.path);
  let inserted_comment =
    Comment::create(&mut context.pool(), &comment_form, parent_path.as_ref()).await?;
  plugin_hook_after("after_create_local_comment", &inserted_comment)?;

  NotifyData::new(
    &post,
    Some(&inserted_comment),
    &local_user_view.person,
    &post_view.community,
    !local_site.disable_email_notifications,
  )
  .send(&context)
  .await?;

  // You like your own comment by default
  let like_form = CommentLikeForm::new(local_user_view.person.id, inserted_comment.id, 1);

  CommentActions::like(&mut context.pool(), &like_form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::CreateComment(inserted_comment.clone()),
    &context,
  )?;

  // Update the read comments, so your own new comment doesn't appear as a +1 unread
  update_read_comments(
    local_user_view.person.id,
    post_id,
    post.comments + 1,
    &mut context.pool(),
  )
  .await?;

  // If we're responding to a comment where we're the recipient,
  // (ie we're the grandparent, or the recipient of the parent comment_reply),
  // then mark the parent as read.
  // Then we don't have to do it manually after we respond to a comment.
  if let Some(parent) = parent_opt {
    let notif = Notification::read_by_comment_id(&mut context.pool(), parent.id).await;
    if let Ok(notif) = notif {
      let person_id = local_user_view.person.id;
      Notification::mark_read_by_id_and_person(&mut context.pool(), notif.id, person_id).await?;
    }
  }

  Ok(Json(
    build_comment_response(
      &context,
      inserted_comment.id,
      Some(local_user_view),
      local_instance_id,
    )
    .await?,
  ))
}
