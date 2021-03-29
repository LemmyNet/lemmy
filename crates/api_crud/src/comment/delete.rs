use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  check_community_ban,
  comment::*,
  get_local_user_view_from_jwt,
  is_mod_or_admin,
  send_local_notifs,
};
use lemmy_apub::ApubObjectType;
use lemmy_db_queries::{source::comment::Comment_, Crud};
use lemmy_db_schema::source::{comment::*, moderator::*};
use lemmy_db_views::comment_view::CommentView;
use lemmy_utils::{ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{messages::SendComment, LemmyContext, UserOperationCrud};

#[async_trait::async_trait(?Send)]
impl PerformCrud for DeleteComment {
  type Response = CommentResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &DeleteComment = &self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let comment_id = data.comment_id;
    let orig_comment = blocking(context.pool(), move |conn| {
      CommentView::read(&conn, comment_id, None)
    })
    .await??;

    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      context.pool(),
    )
    .await?;

    // Verify that only the creator can delete
    if local_user_view.person.id != orig_comment.creator.id {
      return Err(ApiError::err("no_comment_edit_allowed").into());
    }

    // Do the delete
    let deleted = data.deleted;
    let updated_comment = match blocking(context.pool(), move |conn| {
      Comment::update_deleted(conn, comment_id, deleted)
    })
    .await?
    {
      Ok(comment) => comment,
      Err(_e) => return Err(ApiError::err("couldnt_update_comment").into()),
    };

    // Send the apub message
    if deleted {
      updated_comment
        .send_delete(&local_user_view.person, context)
        .await?;
    } else {
      updated_comment
        .send_undo_delete(&local_user_view.person, context)
        .await?;
    }

    // Refetch it
    let comment_id = data.comment_id;
    let person_id = local_user_view.person.id;
    let comment_view = blocking(context.pool(), move |conn| {
      CommentView::read(conn, comment_id, Some(person_id))
    })
    .await??;

    // Build the recipients
    let comment_view_2 = comment_view.clone();
    let mentions = vec![];
    let recipient_ids = send_local_notifs(
      mentions,
      updated_comment,
      local_user_view.person.clone(),
      comment_view_2.post,
      context.pool(),
      false,
    )
    .await?;

    let res = CommentResponse {
      comment_view,
      recipient_ids,
      form_id: None, // TODO a comment delete might clear forms?
    };

    context.chat_server().do_send(SendComment {
      op: UserOperationCrud::DeleteComment,
      comment: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl PerformCrud for RemoveComment {
  type Response = CommentResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &RemoveComment = &self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let comment_id = data.comment_id;
    let orig_comment = blocking(context.pool(), move |conn| {
      CommentView::read(&conn, comment_id, None)
    })
    .await??;

    check_community_ban(
      local_user_view.person.id,
      orig_comment.community.id,
      context.pool(),
    )
    .await?;

    // Verify that only a mod or admin can remove
    is_mod_or_admin(
      context.pool(),
      local_user_view.person.id,
      orig_comment.community.id,
    )
    .await?;

    // Do the remove
    let removed = data.removed;
    let updated_comment = match blocking(context.pool(), move |conn| {
      Comment::update_removed(conn, comment_id, removed)
    })
    .await?
    {
      Ok(comment) => comment,
      Err(_e) => return Err(ApiError::err("couldnt_update_comment").into()),
    };

    // Mod tables
    let form = ModRemoveCommentForm {
      mod_person_id: local_user_view.person.id,
      comment_id: data.comment_id,
      removed: Some(removed),
      reason: data.reason.to_owned(),
    };
    blocking(context.pool(), move |conn| {
      ModRemoveComment::create(conn, &form)
    })
    .await??;

    // Send the apub message
    if removed {
      updated_comment
        .send_remove(&local_user_view.person, context)
        .await?;
    } else {
      updated_comment
        .send_undo_remove(&local_user_view.person, context)
        .await?;
    }

    // Refetch it
    let comment_id = data.comment_id;
    let person_id = local_user_view.person.id;
    let comment_view = blocking(context.pool(), move |conn| {
      CommentView::read(conn, comment_id, Some(person_id))
    })
    .await??;

    // Build the recipients
    let comment_view_2 = comment_view.clone();

    let mentions = vec![];
    let recipient_ids = send_local_notifs(
      mentions,
      updated_comment,
      local_user_view.person.clone(),
      comment_view_2.post,
      context.pool(),
      false,
    )
    .await?;

    let res = CommentResponse {
      comment_view,
      recipient_ids,
      form_id: None, // TODO maybe this might clear other forms
    };

    context.chat_server().do_send(SendComment {
      op: UserOperationCrud::RemoveComment,
      comment: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}
