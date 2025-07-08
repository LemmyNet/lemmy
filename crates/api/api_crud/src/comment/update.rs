use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::{
  build_response::{build_comment_response, send_local_notifs},
  context::LemmyContext,
  plugins::{plugin_hook_after, plugin_hook_before},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_user_action, get_url_blocklist, process_markdown_opt, slur_regex},
};
use lemmy_db_schema::{
  impls::actor_language::validate_post_language,
  newtypes::PostOrCommentId,
  source::comment::{Comment, CommentUpdateForm},
  traits::Crud,
};
use lemmy_db_views_comment::{
  api::{CommentResponse, EditComment},
  CommentView,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  utils::{mention::scrape_text_for_mentions, validation::is_valid_body_field},
};

pub async fn update_comment(
  data: Json<EditComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponse>> {
  let comment_id = data.comment_id;
  let local_instance_id = local_user_view.person.instance_id;
  let orig_comment = CommentView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
    local_instance_id,
  )
  .await?;

  check_community_user_action(
    &local_user_view,
    &orig_comment.community,
    &mut context.pool(),
  )
  .await?;

  // Verify that only the creator can edit
  if local_user_view.person.id != orig_comment.creator.id {
    Err(LemmyErrorType::NoCommentEditAllowed)?
  }

  let language_id = validate_post_language(
    &mut context.pool(),
    data.language_id,
    orig_comment.community.id,
    local_user_view.local_user.id,
  )
  .await?;

  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown_opt(&data.content, &slur_regex, &url_blocklist, &context).await?;
  if let Some(content) = &content {
    is_valid_body_field(content, false)?;
  }

  let comment_id = data.comment_id;
  let mut form = CommentUpdateForm {
    content,
    language_id: Some(language_id),
    updated_at: Some(Some(Utc::now())),
    ..Default::default()
  };
  form = plugin_hook_before("before_update_local_comment", form).await?;
  let updated_comment = Comment::update(&mut context.pool(), comment_id, &form).await?;

  plugin_hook_after("after_update_local_comment", &updated_comment)?;

  // Do the mentions / recipients
  let updated_comment_content = updated_comment.content.clone();
  let mentions = scrape_text_for_mentions(&updated_comment_content);
  let recipient_ids = send_local_notifs(
    mentions,
    PostOrCommentId::Comment(comment_id),
    &local_user_view.person,
    false,
    &context,
    Some(&local_user_view),
    local_instance_id,
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::UpdateComment(updated_comment.clone()),
    &context,
  )?;

  Ok(Json(
    build_comment_response(
      &context,
      updated_comment.id,
      Some(local_user_view),
      recipient_ids,
      local_instance_id,
    )
    .await?,
  ))
}
