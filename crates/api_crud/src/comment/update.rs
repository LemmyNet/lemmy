use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{CommentResponse, EditComment},
  utils::{
    blocking,
    check_community_ban,
    check_community_deleted_or_removed,
    check_post_deleted_or_removed,
    get_local_user_view_from_jwt,
    is_mod_or_admin,
    local_site_to_slur_regex,
  },
};
use lemmy_apub::protocol::activities::{
  create_or_update::comment::CreateOrUpdateComment,
  CreateOrUpdateType,
};
use lemmy_db_schema::{
  source::{
    actor_language::CommunityLanguage,
    comment::{Comment, CommentUpdateForm},
    local_site::LocalSite,
  },
  traits::Crud,
};
use lemmy_db_views::structs::CommentView;
use lemmy_utils::{
  error::LemmyError,
  utils::{remove_slurs, scrape_text_for_mentions},
  ConnectionId,
};
use lemmy_websocket::{
  send::{send_comment_ws_message, send_local_notifs},
  LemmyContext,
  UserOperationCrud,
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
    let local_site = blocking(context.pool(), LocalSite::read).await??;

    let comment_id = data.comment_id;
    let orig_comment = blocking(context.pool(), move |conn| {
      CommentView::read(conn, comment_id, None)
    })
    .await??;

    // TODO is this necessary? It should really only need to check on create
    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      context.pool(),
    )
    .await?;
    check_community_deleted_or_removed(orig_comment.community.id, context.pool()).await?;
    check_post_deleted_or_removed(&orig_comment.post)?;

    // Verify that only the creator can edit
    if local_user_view.person.id != orig_comment.creator.id {
      return Err(LemmyError::from_message("no_comment_edit_allowed"));
    }

    if data.distinguished.is_some() {
      // Verify that only a mod or admin can distinguish a comment
      is_mod_or_admin(
        context.pool(),
        local_user_view.person.id,
        orig_comment.community.id,
      )
      .await?;
    }

    let language_id = self.language_id;
    blocking(context.pool(), move |conn| {
      CommunityLanguage::is_allowed_community_language(conn, language_id, orig_comment.community.id)
    })
    .await??;

    // Update the Content
    let content_slurs_removed = data
      .content
      .as_ref()
      .map(|c| remove_slurs(c, &local_site_to_slur_regex(&local_site)));
    let comment_id = data.comment_id;
    let form = CommentUpdateForm::builder()
      .content(content_slurs_removed)
      .distinguished(data.distinguished)
      .language_id(data.language_id)
      .build();
    let updated_comment = blocking(context.pool(), move |conn| {
      Comment::update(conn, comment_id, &form)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_comment"))?;

    // Do the mentions / recipients
    let updated_comment_content = updated_comment.content.to_owned();
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

    // Send the apub update
    CreateOrUpdateComment::send(
      updated_comment.into(),
      &local_user_view.person.into(),
      CreateOrUpdateType::Update,
      context,
      &mut 0,
    )
    .await?;

    send_comment_ws_message(
      data.comment_id,
      UserOperationCrud::EditComment,
      websocket_id,
      data.form_id.to_owned(),
      None,
      recipient_ids,
      context,
    )
    .await
  }
}
