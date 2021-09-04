use crate::{
  messages::{SendComment, SendCommunityRoomMessage, SendPost, SendUserRoomMessage},
  LemmyContext,
  OperationType,
};
use lemmy_api_common::{
  blocking,
  comment::CommentResponse,
  community::CommunityResponse,
  person::PrivateMessageResponse,
  post::PostResponse,
};
use lemmy_db_queries::DeleteableOrRemoveable;
use lemmy_db_schema::{CommentId, CommunityId, LocalUserId, PersonId, PostId, PrivateMessageId};
use lemmy_db_views::{
  comment_view::CommentView,
  local_user_view::LocalUserView,
  post_view::PostView,
  private_message_view::PrivateMessageView,
};
use lemmy_db_views_actor::community_view::CommunityView;
use lemmy_utils::{ConnectionId, LemmyError};

pub async fn send_post_ws_message<OP: ToString + Send + OperationType + 'static>(
  post_id: PostId,
  op: OP,
  websocket_id: Option<ConnectionId>,
  person_id: Option<PersonId>,
  context: &LemmyContext,
) -> Result<PostResponse, LemmyError> {
  let mut post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, post_id, person_id)
  })
  .await??;

  if post_view.post.deleted || post_view.post.removed {
    post_view.post = post_view.post.blank_out_deleted_or_removed_info();
  }

  let res = PostResponse { post_view };

  context.chat_server().do_send(SendPost {
    op,
    post: res.clone(),
    websocket_id,
  });

  Ok(res)
}

// TODO: in many call sites in apub crate, we are setting an empty vec for recipient_ids,
//       we should get the actual recipient actors from somewhere
pub async fn send_comment_ws_message_simple<OP: ToString + Send + OperationType + 'static>(
  comment_id: CommentId,
  op: OP,
  context: &LemmyContext,
) -> Result<CommentResponse, LemmyError> {
  send_comment_ws_message(comment_id, op, None, None, None, vec![], context).await
}

pub async fn send_comment_ws_message<OP: ToString + Send + OperationType + 'static>(
  comment_id: CommentId,
  op: OP,
  websocket_id: Option<ConnectionId>,
  form_id: Option<String>,
  person_id: Option<PersonId>,
  recipient_ids: Vec<LocalUserId>,
  context: &LemmyContext,
) -> Result<CommentResponse, LemmyError> {
  let mut view = blocking(context.pool(), move |conn| {
    CommentView::read(conn, comment_id, person_id)
  })
  .await??;

  if view.comment.deleted || view.comment.removed {
    view.comment = view.comment.blank_out_deleted_or_removed_info();
  }

  let mut res = CommentResponse {
    comment_view: view,
    recipient_ids,
    // The sent out form id should be null
    form_id: None,
  };

  context.chat_server().do_send(SendComment {
    op,
    comment: res.clone(),
    websocket_id,
  });

  // The recipient_ids should be empty for returns
  res.recipient_ids = Vec::new();
  res.form_id = form_id;

  Ok(res)
}

pub async fn send_community_ws_message<OP: ToString + Send + OperationType + 'static>(
  community_id: CommunityId,
  op: OP,
  websocket_id: Option<ConnectionId>,
  person_id: Option<PersonId>,
  context: &LemmyContext,
) -> Result<CommunityResponse, LemmyError> {
  let mut community_view = blocking(context.pool(), move |conn| {
    CommunityView::read(conn, community_id, person_id)
  })
  .await??;
  // Blank out deleted or removed info
  if community_view.community.deleted || community_view.community.removed {
    community_view.community = community_view.community.blank_out_deleted_or_removed_info();
  }

  let res = CommunityResponse { community_view };

  // Strip out the person id and subscribed when sending to others
  let mut res_mut = res.clone();
  res_mut.community_view.subscribed = false;

  context.chat_server().do_send(SendCommunityRoomMessage {
    op,
    response: res_mut,
    community_id: res.community_view.community.id,
    websocket_id,
  });

  Ok(res)
}

pub async fn send_pm_ws_message<OP: ToString + Send + OperationType + 'static>(
  private_message_id: PrivateMessageId,
  op: OP,
  websocket_id: Option<ConnectionId>,
  context: &LemmyContext,
) -> Result<PrivateMessageResponse, LemmyError> {
  let mut view = blocking(context.pool(), move |conn| {
    PrivateMessageView::read(conn, private_message_id)
  })
  .await??;

  // Blank out deleted or removed info
  if view.private_message.deleted {
    view.private_message = view.private_message.blank_out_deleted_or_removed_info();
  }

  let res = PrivateMessageResponse {
    private_message_view: view,
  };

  // Send notifications to the local recipient, if one exists
  if res.private_message_view.recipient.local {
    let recipient_id = res.private_message_view.recipient.id;
    let local_recipient = blocking(context.pool(), move |conn| {
      LocalUserView::read_person(conn, recipient_id)
    })
    .await??;
    context.chat_server().do_send(SendUserRoomMessage {
      op,
      response: res.clone(),
      local_recipient_id: local_recipient.local_user.id,
      websocket_id,
    });
  }

  Ok(res)
}
