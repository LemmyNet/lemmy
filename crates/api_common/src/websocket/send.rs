use super::{
  handlers::messages::{
    SendAllMessage,
    SendCommunityRoomMessage,
    SendModRoomMessage,
    SendPostRoomMessage,
    SendUserRoomMessage,
  },
  serialize_websocket_message,
};
use crate::{
  comment::CommentResponse,
  community::CommunityResponse,
  context::LemmyContext,
  post::PostResponse,
  private_message::PrivateMessageResponse,
  utils::{check_person_block, get_interface_language, send_email_to_user},
};
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, LocalUserId, PersonId, PostId, PrivateMessageId},
  source::{
    actor_language::CommunityLanguage,
    comment::Comment,
    comment_reply::{CommentReply, CommentReplyInsertForm},
    person::Person,
    person_mention::{PersonMention, PersonMentionInsertForm},
    post::Post,
  },
  traits::Crud,
  SubscribedType,
};
use lemmy_db_views::structs::{CommentView, LocalUserView, PostView, PrivateMessageView};
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::{error::LemmyError, utils::mention::MentionData, ConnectionId};
use serde::Serialize;

impl LemmyContext {
  #[tracing::instrument(skip_all)]
  pub async fn send_post_ws_message<OP>(
    &self,
    op: &OP,
    post_id: PostId,
    websocket_id: Option<ConnectionId>,
    person_id: Option<PersonId>,
  ) -> Result<PostResponse, LemmyError>
  where
    OP: ToString,
  {
    let post_view = PostView::read(self.pool(), post_id, person_id, Some(true)).await?;

    let res = PostResponse { post_view };

    // Send it to the post room
    // Don't send my data with it
    let mut post_sent = res.clone();
    post_sent.post_view.my_vote = None;
    let message = serialize_websocket_message(op, &post_sent)?;

    self.chat_server().do_send(SendPostRoomMessage {
      post_id,
      message: message.clone(),
      websocket_id,
    });

    // Send it to /c/all and that community
    self.chat_server().do_send(SendCommunityRoomMessage {
      community_id: CommunityId(0),
      message: message.clone(),
      websocket_id,
    });

    self.chat_server().do_send(SendCommunityRoomMessage {
      community_id: post_sent.post_view.community.id,
      message,
      websocket_id,
    });

    Ok(res)
  }

  // TODO: in many call sites in apub crate, we are setting an empty vec for recipient_ids,
  //       we should get the actual recipient actors from somewhere
  #[tracing::instrument(skip_all)]
  pub async fn send_comment_ws_message_simple<OP>(
    &self,
    op: &OP,
    comment_id: CommentId,
  ) -> Result<CommentResponse, LemmyError>
  where
    OP: ToString,
  {
    self
      .send_comment_ws_message(op, comment_id, None, None, None, vec![])
      .await
  }

  #[tracing::instrument(skip_all)]
  pub async fn send_comment_ws_message<OP>(
    &self,
    op: &OP,
    comment_id: CommentId,
    websocket_id: Option<ConnectionId>,
    form_id: Option<String>,
    person_id: Option<PersonId>,
    recipient_ids: Vec<LocalUserId>,
  ) -> Result<CommentResponse, LemmyError>
  where
    OP: ToString,
  {
    let view = CommentView::read(self.pool(), comment_id, person_id).await?;

    let mut res = CommentResponse {
      comment_view: view,
      recipient_ids,
      form_id,
    };

    // Strip out my specific user info
    let mut sent_recipient_comment = res.clone();
    sent_recipient_comment.form_id = None;
    sent_recipient_comment.comment_view.my_vote = None;
    let recipient_message = serialize_websocket_message(op, &sent_recipient_comment)?;

    // Send it to the recipient(s) including the mentioned users
    for recipient_id in &sent_recipient_comment.recipient_ids {
      self.chat_server().do_send(SendUserRoomMessage {
        recipient_id: *recipient_id,
        message: recipient_message.clone(),
        websocket_id,
      });
    }

    // Remove the recipients here to separate mentions / user messages from post or community comments
    let mut sent_post_comment = sent_recipient_comment;
    sent_post_comment.recipient_ids = Vec::new();
    let post_message = serialize_websocket_message(op, &sent_post_comment)?;

    // Send it to the post room
    self.chat_server().do_send(SendPostRoomMessage {
      post_id: sent_post_comment.comment_view.post.id,
      message: post_message.clone(),
      websocket_id,
    });

    // Send it to the community too
    self.chat_server().do_send(SendCommunityRoomMessage {
      community_id: sent_post_comment.comment_view.community.id,
      message: post_message,
      websocket_id,
    });
    // TODO should I send it to all? Seems excessive
    //  self
    //    .send_community_room_message(
    //      user_operation,
    //      &comment_post_sent,
    //      CommunityId(0),
    //      websocket_id,
    //    )
    //    .await?;

    // No need to return recipients
    res.recipient_ids = Vec::new();

    Ok(res)
  }

  #[tracing::instrument(skip_all)]
  pub async fn send_community_ws_message<OP>(
    &self,
    op: &OP,
    community_id: CommunityId,
    websocket_id: Option<ConnectionId>,
    person_id: Option<PersonId>,
  ) -> Result<CommunityResponse, LemmyError>
  where
    OP: ToString,
  {
    let community_view =
      CommunityView::read(self.pool(), community_id, person_id, Some(true)).await?;
    let discussion_languages = CommunityLanguage::read(self.pool(), community_id).await?;

    let mut res = CommunityResponse {
      community_view,
      discussion_languages,
    };

    // Strip out the person id and subscribed when sending to others
    res.community_view.subscribed = SubscribedType::NotSubscribed;
    let message = serialize_websocket_message(op, &res)?;

    self.chat_server().do_send(SendCommunityRoomMessage {
      community_id: res.community_view.community.id,
      message,
      websocket_id,
    });

    Ok(res)
  }

  #[tracing::instrument(skip_all)]
  pub async fn send_pm_ws_message<OP>(
    &self,
    op: &OP,
    private_message_id: PrivateMessageId,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PrivateMessageResponse, LemmyError>
  where
    OP: ToString,
  {
    let view = PrivateMessageView::read(self.pool(), private_message_id).await?;

    let res = PrivateMessageResponse {
      private_message_view: view,
    };

    // Send notifications to the local recipient, if one exists
    if res.private_message_view.recipient.local {
      let recipient_id = res.private_message_view.recipient.id;
      let local_recipient = LocalUserView::read_person(self.pool(), recipient_id).await?;

      let message = serialize_websocket_message(op, &res)?;

      self.chat_server().do_send(SendUserRoomMessage {
        recipient_id: local_recipient.local_user.id,
        message,
        websocket_id,
      });
    }

    Ok(res)
  }

  #[tracing::instrument(skip_all)]
  pub async fn send_local_notifs(
    &self,
    mentions: Vec<MentionData>,
    comment: &Comment,
    person: &Person,
    post: &Post,
    do_send_email: bool,
  ) -> Result<Vec<LocalUserId>, LemmyError> {
    let mut recipient_ids = Vec::new();
    let inbox_link = format!("{}/inbox", self.settings().get_protocol_and_hostname());

    // Send the local mentions
    for mention in mentions
      .iter()
      .filter(|m| m.is_local(&self.settings().hostname) && m.name.ne(&person.name))
      .collect::<Vec<&MentionData>>()
    {
      let mention_name = mention.name.clone();
      let user_view = LocalUserView::read_from_name(self.pool(), &mention_name).await;
      if let Ok(mention_user_view) = user_view {
        // TODO
        // At some point, make it so you can't tag the parent creator either
        // This can cause two notifications, one for reply and the other for mention
        recipient_ids.push(mention_user_view.local_user.id);

        let user_mention_form = PersonMentionInsertForm {
          recipient_id: mention_user_view.person.id,
          comment_id: comment.id,
          read: None,
        };

        // Allow this to fail softly, since comment edits might re-update or replace it
        // Let the uniqueness handle this fail
        PersonMention::create(self.pool(), &user_mention_form)
          .await
          .ok();

        // Send an email to those local users that have notifications on
        if do_send_email {
          let lang = get_interface_language(&mention_user_view);
          send_email_to_user(
            &mention_user_view,
            &lang.notification_mentioned_by_subject(&person.name),
            &lang.notification_mentioned_by_body(&comment.content, &inbox_link, &person.name),
            self.settings(),
          )
        }
      }
    }

    // Send comment_reply to the parent commenter / poster
    if let Some(parent_comment_id) = comment.parent_comment_id() {
      let parent_comment = Comment::read(self.pool(), parent_comment_id).await?;

      // Get the parent commenter local_user
      let parent_creator_id = parent_comment.creator_id;

      // Only add to recipients if that person isn't blocked
      let creator_blocked = check_person_block(person.id, parent_creator_id, self.pool())
        .await
        .is_err();

      // Don't send a notif to yourself
      if parent_comment.creator_id != person.id && !creator_blocked {
        let user_view = LocalUserView::read_person(self.pool(), parent_creator_id).await;
        if let Ok(parent_user_view) = user_view {
          recipient_ids.push(parent_user_view.local_user.id);

          let comment_reply_form = CommentReplyInsertForm {
            recipient_id: parent_user_view.person.id,
            comment_id: comment.id,
            read: None,
          };

          // Allow this to fail softly, since comment edits might re-update or replace it
          // Let the uniqueness handle this fail
          CommentReply::create(self.pool(), &comment_reply_form)
            .await
            .ok();

          if do_send_email {
            let lang = get_interface_language(&parent_user_view);
            send_email_to_user(
              &parent_user_view,
              &lang.notification_comment_reply_subject(&person.name),
              &lang.notification_comment_reply_body(&comment.content, &inbox_link, &person.name),
              self.settings(),
            )
          }
        }
      }
    } else {
      // If there's no parent, its the post creator
      // Only add to recipients if that person isn't blocked
      let creator_blocked = check_person_block(person.id, post.creator_id, self.pool())
        .await
        .is_err();

      if post.creator_id != person.id && !creator_blocked {
        let creator_id = post.creator_id;
        let parent_user = LocalUserView::read_person(self.pool(), creator_id).await;
        if let Ok(parent_user_view) = parent_user {
          recipient_ids.push(parent_user_view.local_user.id);

          let comment_reply_form = CommentReplyInsertForm {
            recipient_id: parent_user_view.person.id,
            comment_id: comment.id,
            read: None,
          };

          // Allow this to fail softly, since comment edits might re-update or replace it
          // Let the uniqueness handle this fail
          CommentReply::create(self.pool(), &comment_reply_form)
            .await
            .ok();

          if do_send_email {
            let lang = get_interface_language(&parent_user_view);
            send_email_to_user(
              &parent_user_view,
              &lang.notification_post_reply_subject(&person.name),
              &lang.notification_post_reply_body(&comment.content, &inbox_link, &person.name),
              self.settings(),
            )
          }
        }
      }
    }

    Ok(recipient_ids)
  }

  #[tracing::instrument(skip_all)]
  pub fn send_all_ws_message<Data, OP>(
    &self,
    op: &OP,
    data: Data,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    Data: Serialize,
    OP: ToString,
  {
    let message = serialize_websocket_message(op, &data)?;
    self.chat_server().do_send(SendAllMessage {
      message,
      websocket_id,
    });
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  pub fn send_mod_ws_message<Data, OP>(
    &self,
    op: &OP,
    data: Data,
    community_id: CommunityId,
    websocket_id: Option<ConnectionId>,
  ) -> Result<(), LemmyError>
  where
    Data: Serialize,
    OP: ToString,
  {
    let message = serialize_websocket_message(op, &data)?;
    self.chat_server().do_send(SendModRoomMessage {
      community_id,
      message,
      websocket_id,
    });
    Ok(())
  }
}
