use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::{build_comment_response, send_local_notifs},
  comment::{CommentResponse, EditComment},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_community_user_action,
    get_url_blocklist,
    local_site_to_slur_regex,
    process_markdown_opt,
  },
};
use lemmy_db_schema::{
  source::{
    actor_language::CommunityLanguage,
    comment::{Comment, CommentUpdateForm},
    local_site::LocalSite,
  },
  traits::Crud,
  utils::naive_now,
};
use lemmy_db_views::structs::{CommentView, LocalUserView};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::{mention::scrape_text_for_mentions, validation::is_valid_body_field},
};

#[tracing::instrument(skip(context))]
pub async fn update_comment(
  data: Json<EditComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  let comment_id = data.comment_id;
  let orig_comment = CommentView::read(&mut context.pool(), comment_id, None)
    .await?
    .ok_or(LemmyErrorType::CouldntFindComment)?;

  check_community_user_action(
    &local_user_view.person,
    orig_comment.community.id,
    &mut context.pool(),
  )
  .await?;

  // Verify that only the creator can edit
  if local_user_view.person.id != orig_comment.creator.id {
    Err(LemmyErrorType::NoCommentEditAllowed)?
  }

  let language_id = data.language_id;
  CommunityLanguage::is_allowed_community_language(
    &mut context.pool(),
    language_id,
    orig_comment.community.id,
  )
  .await?;

  let slur_regex = local_site_to_slur_regex(&local_site);
  let url_blocklist = get_url_blocklist(&context).await?;
  let content = process_markdown_opt(&data.content, &slur_regex, &url_blocklist, &context).await?;
  if let Some(content) = &content {
    is_valid_body_field(content, false)?;
  }

  let comment_id = data.comment_id;
  let form = CommentUpdateForm {
    content,
    language_id: data.language_id,
    updated: Some(Some(naive_now())),
    ..Default::default()
  };
  let updated_comment = Comment::update(&mut context.pool(), comment_id, &form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateComment)?;

  // Do the mentions / recipients
  let updated_comment_content = updated_comment.content.clone();
  let mentions = scrape_text_for_mentions(&updated_comment_content);
  let recipient_ids = send_local_notifs(
    mentions,
    comment_id,
    &local_user_view.person,
    false,
    &context,
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::UpdateComment(updated_comment.clone()),
    &context,
  )
  .await?;

  Ok(Json(
    build_comment_response(
      &context,
      updated_comment.id,
      Some(local_user_view),
      recipient_ids,
    )
    .await?,
  ))
}
