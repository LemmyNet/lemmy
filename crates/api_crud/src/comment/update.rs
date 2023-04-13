use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{CommentResponse, EditComment},
  context::LemmyContext,
  utils::{check_community_ban, get_local_user_view_from_jwt, local_site_to_slur_regex},
  websocket::UserOperationCrud,
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
  ConnectionId,
};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &EditComment = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;
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
    let content_slurs_removed = data
      .content
      .as_ref()
      .map(|c| remove_slurs(c, &local_site_to_slur_regex(&local_site)));

    is_valid_body_field(&content_slurs_removed)?;

    let comment_id = data.comment_id;
    let form = CommentUpdateForm::builder()
      .content(content_slurs_removed)
      .language_id(data.language_id)
      .updated(Some(Some(naive_now())))
      .build();
    let updated_comment = Comment::update(context.pool(), comment_id, &form)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_comment"))?;

    // Do the mentions / recipients
    let updated_comment_content = updated_comment.content.clone();
    let mentions = scrape_text_for_mentions(&updated_comment_content);
    let recipient_ids = context
      .send_local_notifs(
        mentions,
        &updated_comment,
        &local_user_view.person,
        &orig_comment.post,
        false,
      )
      .await?;

    context
      .send_comment_ws_message(
        &UserOperationCrud::EditComment,
        data.comment_id,
        websocket_id,
        data.form_id.clone(),
        None,
        recipient_ids,
      )
      .await
  }
}
