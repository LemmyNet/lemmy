#[cfg(feature = "full")]
pub mod build_response;
pub mod comment;
pub mod community;
#[cfg(feature = "full")]
pub mod context;
pub mod custom_emoji;
pub mod person;
pub mod post;
pub mod private_message;
#[cfg(feature = "full")]
pub mod request;
pub mod sensitive;
pub mod site;
#[cfg(feature = "full")]
pub mod utils;

pub extern crate lemmy_db_schema;
pub extern crate lemmy_db_views;
pub extern crate lemmy_db_views_actor;
pub extern crate lemmy_db_views_moderator;

use crate::{
  context::LemmyContext,
  utils::{check_person_block, get_interface_language, send_email_to_user},
};
use lemmy_db_schema::{
  newtypes::LocalUserId,
  source::{
    comment::Comment,
    comment_reply::{CommentReply, CommentReplyInsertForm},
    person::Person,
    person_mention::{PersonMention, PersonMentionInsertForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{error::LemmyError, utils::mention::MentionData};

// TODO: this function is a mess and should be split up to handle email seperately
#[tracing::instrument(skip_all)]
pub async fn send_local_notifs(
  mentions: Vec<MentionData>,
  comment: &Comment,
  person: &Person,
  post: &Post,
  do_send_email: bool,
  context: &LemmyContext,
) -> Result<Vec<LocalUserId>, LemmyError> {
  let mut recipient_ids = Vec::new();
  let inbox_link = format!("{}/inbox", context.settings().get_protocol_and_hostname());

  // Send the local mentions
  for mention in mentions
    .iter()
    .filter(|m| m.is_local(&context.settings().hostname) && m.name.ne(&person.name))
    .collect::<Vec<&MentionData>>()
  {
    let mention_name = mention.name.clone();
    let user_view = LocalUserView::read_from_name(context.pool(), &mention_name).await;
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
      PersonMention::create(context.pool(), &user_mention_form)
        .await
        .ok();

      // Send an email to those local users that have notifications on
      if do_send_email {
        let lang = get_interface_language(&mention_user_view);
        send_email_to_user(
          &mention_user_view,
          &lang.notification_mentioned_by_subject(&person.name),
          &lang.notification_mentioned_by_body(&comment.content, &inbox_link, &person.name),
          context.settings(),
        )
      }
    }
  }

  // Send comment_reply to the parent commenter / poster
  if let Some(parent_comment_id) = comment.parent_comment_id() {
    let parent_comment = Comment::read(context.pool(), parent_comment_id).await?;

    // Get the parent commenter local_user
    let parent_creator_id = parent_comment.creator_id;

    // Only add to recipients if that person isn't blocked
    let creator_blocked = check_person_block(person.id, parent_creator_id, context.pool())
      .await
      .is_err();

    // Don't send a notif to yourself
    if parent_comment.creator_id != person.id && !creator_blocked {
      let user_view = LocalUserView::read_person(context.pool(), parent_creator_id).await;
      if let Ok(parent_user_view) = user_view {
        recipient_ids.push(parent_user_view.local_user.id);

        let comment_reply_form = CommentReplyInsertForm {
          recipient_id: parent_user_view.person.id,
          comment_id: comment.id,
          read: None,
        };

        // Allow this to fail softly, since comment edits might re-update or replace it
        // Let the uniqueness handle this fail
        CommentReply::create(context.pool(), &comment_reply_form)
          .await
          .ok();

        if do_send_email {
          let lang = get_interface_language(&parent_user_view);
          send_email_to_user(
            &parent_user_view,
            &lang.notification_comment_reply_subject(&person.name),
            &lang.notification_comment_reply_body(&comment.content, &inbox_link, &person.name),
            context.settings(),
          )
        }
      }
    }
  } else {
    // If there's no parent, its the post creator
    // Only add to recipients if that person isn't blocked
    let creator_blocked = check_person_block(person.id, post.creator_id, context.pool())
      .await
      .is_err();

    if post.creator_id != person.id && !creator_blocked {
      let creator_id = post.creator_id;
      let parent_user = LocalUserView::read_person(context.pool(), creator_id).await;
      if let Ok(parent_user_view) = parent_user {
        recipient_ids.push(parent_user_view.local_user.id);

        let comment_reply_form = CommentReplyInsertForm {
          recipient_id: parent_user_view.person.id,
          comment_id: comment.id,
          read: None,
        };

        // Allow this to fail softly, since comment edits might re-update or replace it
        // Let the uniqueness handle this fail
        CommentReply::create(context.pool(), &comment_reply_form)
          .await
          .ok();

        if do_send_email {
          let lang = get_interface_language(&parent_user_view);
          send_email_to_user(
            &parent_user_view,
            &lang.notification_post_reply_subject(&person.name),
            &lang.notification_post_reply_body(&comment.content, &inbox_link, &person.name),
            context.settings(),
          )
        }
      }
    }
  }

  Ok(recipient_ids)
}
