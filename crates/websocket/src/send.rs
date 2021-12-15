use crate::{
  messages::{SendComment, SendCommunityRoomMessage, SendPost, SendUserRoomMessage},
  LemmyContext,
  OperationType,
};
use lemmy_api_common::{
  blocking,
  check_person_block,
  comment::CommentResponse,
  community::CommunityResponse,
  person::PrivateMessageResponse,
  post::PostResponse,
  send_email_to_user,
};
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, LocalUserId, PersonId, PostId, PrivateMessageId},
  source::{
    comment::Comment,
    person::Person,
    person_mention::{PersonMention, PersonMentionForm},
    post::Post,
  },
  traits::{Crud, DeleteableOrRemoveable},
};
use lemmy_db_views::{
  comment_view::CommentView,
  local_user_view::LocalUserView,
  post_view::PostView,
  private_message_view::PrivateMessageView,
};
use lemmy_db_views_actor::community_view::CommunityView;
use lemmy_utils::{utils::MentionData, ConnectionId, LemmyError};

pub async fn send_post_ws_message<OP: ToString + Send + OperationType + 'static>(
  post_id: PostId,
  op: OP,
  websocket_id: Option<ConnectionId>,
  person_id: Option<PersonId>,
  context: &LemmyContext,
) -> Result<PostResponse, LemmyError> {
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, post_id, person_id)
  })
  .await??;

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
  let community_view = blocking(context.pool(), move |conn| {
    CommunityView::read(conn, community_id, person_id)
  })
  .await??;

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

pub async fn send_local_notifs(
  mentions: Vec<MentionData>,
  comment: &Comment,
  person: &Person,
  post: &Post,
  do_send_email: bool,
  context: &LemmyContext,
) -> Result<Vec<LocalUserId>, LemmyError> {
  let mut recipient_ids = Vec::new();

  // Send the local mentions
  for mention in mentions
    .iter()
    .filter(|m| m.is_local(&context.settings().hostname) && m.name.ne(&person.name))
    .collect::<Vec<&MentionData>>()
  {
    let mention_name = mention.name.clone();
    let user_view = blocking(context.pool(), move |conn| {
      LocalUserView::read_from_name(conn, &mention_name)
    })
    .await?;
    if let Ok(mention_user_view) = user_view {
      // TODO
      // At some point, make it so you can't tag the parent creator either
      // This can cause two notifications, one for reply and the other for mention
      recipient_ids.push(mention_user_view.local_user.id);

      let user_mention_form = PersonMentionForm {
        recipient_id: mention_user_view.person.id,
        comment_id: comment.id,
        read: None,
      };

      // Allow this to fail softly, since comment edits might re-update or replace it
      // Let the uniqueness handle this fail
      blocking(context.pool(), move |conn| {
        PersonMention::create(conn, &user_mention_form)
      })
      .await?
      .ok();

      // Send an email to those local users that have notifications on
      if do_send_email {
        send_email_to_user(
          &mention_user_view,
          "Mentioned by",
          "Person Mention",
          &comment.content,
          &context.settings(),
        )
      }
    }
  }

  // Send notifs to the parent commenter / poster
  match comment.parent_id {
    Some(parent_id) => {
      let parent_comment =
        blocking(context.pool(), move |conn| Comment::read(conn, parent_id)).await?;
      if let Ok(parent_comment) = parent_comment {
        // Get the parent commenter local_user
        let parent_creator_id = parent_comment.creator_id;

        // Only add to recipients if that person isn't blocked
        let creator_blocked = check_person_block(person.id, parent_creator_id, context.pool())
          .await
          .is_err();

        // Don't send a notif to yourself
        if parent_comment.creator_id != person.id && !creator_blocked {
          let user_view = blocking(context.pool(), move |conn| {
            LocalUserView::read_person(conn, parent_creator_id)
          })
          .await?;
          if let Ok(parent_user_view) = user_view {
            recipient_ids.push(parent_user_view.local_user.id);

            if do_send_email {
              send_email_to_user(
                &parent_user_view,
                "Reply from",
                "Comment Reply",
                &comment.content,
                &context.settings(),
              )
            }
          }
        }
      }
    }
    // Its a post
    // Don't send a notif to yourself
    None => {
      // Only add to recipients if that person isn't blocked
      let creator_blocked = check_person_block(person.id, post.creator_id, context.pool())
        .await
        .is_err();

      if post.creator_id != person.id && !creator_blocked {
        let creator_id = post.creator_id;
        let parent_user = blocking(context.pool(), move |conn| {
          LocalUserView::read_person(conn, creator_id)
        })
        .await?;
        if let Ok(parent_user_view) = parent_user {
          recipient_ids.push(parent_user_view.local_user.id);

          if do_send_email {
            send_email_to_user(
              &parent_user_view,
              "Reply from",
              "Post Reply",
              &comment.content,
              &context.settings(),
            )
          }
        }
      }
    }
  };
  Ok(recipient_ids)
}
