use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::{build_comment_response, send_local_notifs},
  comment::{CommentResponse, EditComment},
  context::LemmyContext,
  utils::{
    check_community_ban,
    local_site_to_slur_regex,
    local_user_view_from_jwt,
    sanitize_html_opt,
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
use lemmy_db_views::structs::CommentView;
use lemmy_utils::{
  error::LemmyError,
  utils::{
    mention::scrape_text_for_mentions,
    slurs::remove_slurs,
    validation::is_valid_body_field,
  },
};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CommentResponse, LemmyError> {
    let data: &EditComment = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;
    let local_site = LocalSite::read(context.pool()).await?;

    let comment_id = data.comment_id;
    let orig_comment = CommentView::read(context.pool(), comment_id, None).await?;

    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      context.pool(),
    )
    .await?;

    // Verify that only the creator can edit
    if local_user_view.person.id != orig_comment.creator.id {
      return Err(LemmyError::from_message("no_comment_edit_allowed"));
    }

    let language_id = self.language_id;
    CommunityLanguage::is_allowed_community_language(
      context.pool(),
      language_id,
      orig_comment.community.id,
    )
    .await?;

    // Update the Content
    let content = data
      .content
      .as_ref()
      .map(|c| remove_slurs(c, &local_site_to_slur_regex(&local_site)));
    is_valid_body_field(&content, false)?;
    let content = sanitize_html_opt(&content);

    let comment_id = data.comment_id;
    let form = CommentUpdateForm::builder()
      .content(content)
      .language_id(data.language_id)
      .updated(Some(Some(naive_now())))
      .build();
    let updated_comment = Comment::update(context.pool(), comment_id, &form)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_comment"))?;

    // Do the mentions / recipients
    let updated_comment_content = updated_comment.content.clone();
    let mentions = scrape_text_for_mentions(&updated_comment_content);
    let recipient_ids = send_local_notifs(
      mentions,
      &updated_comment,
      &local_user_view.person,
      &orig_comment.post,
      false,
      context,
    )
    .await?;

    build_comment_response(
      context,
      updated_comment.id,
      Some(local_user_view),
      self.form_id.clone(),
      recipient_ids,
    )
    .await
  }
}
