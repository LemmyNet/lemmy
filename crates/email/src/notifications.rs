use crate::{inbox_link, send::send_email, user_language};
use lemmy_db_schema::{
  newtypes::DbUrl,
  source::{comment::Comment, community::Community, person::Person, post::Post},
};
use lemmy_db_schema_file::enums::ModlogKind;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{settings::structs::Settings, utils::markdown::markdown_to_html};

pub enum NotificationEmailData<'a> {
  Mention {
    content: String,
    person: &'a Person,
  },
  PostSubscribed {
    post: &'a Post,
    comment: &'a Comment,
  },
  CommunitySubscribed {
    post: &'a Post,
    community: &'a Community,
  },
  Reply {
    comment: &'a Comment,
    person: &'a Person,
    parent_comment: Option<Comment>,
    post: &'a Post,
  },
  PrivateMessage {
    sender: &'a Person,
    content: &'a String,
  },
  ModAction {
    kind: ModlogKind,
    reason: Option<&'a str>,
    is_revert: bool,
  },
}

pub fn send_notification_email(
  local_user_view: LocalUserView,
  link: DbUrl,
  data: NotificationEmailData,
  settings: &'static Settings,
) {
  if local_user_view.banned || !local_user_view.local_user.send_notifications_to_email {
    return;
  }

  let inbox_link = inbox_link(settings);
  let lang = user_language(&local_user_view.local_user);
  let (subject, body) = match data {
    NotificationEmailData::Mention { content, person } => {
      let content = markdown_to_html(&content);
      (
        lang.notification_mentioned_by_subject(&person.name),
        lang.notification_mentioned_by_body(&link, &content, &inbox_link, &person.name),
      )
    }
    NotificationEmailData::PostSubscribed { post, comment } => {
      let content = markdown_to_html(&comment.content);
      (
        lang.notification_post_subscribed_subject(&post.name),
        lang.notification_post_subscribed_body(&content, &link, inbox_link),
      )
    }
    NotificationEmailData::CommunitySubscribed { post, community } => {
      let content = post
        .body
        .as_ref()
        .map(|b| markdown_to_html(b))
        .unwrap_or_default();
      (
        lang.notification_community_subscribed_subject(&post.name, &community.title),
        lang.notification_community_subscribed_body(&content, &link, inbox_link),
      )
    }
    NotificationEmailData::Reply {
      comment,
      person,
      parent_comment: Some(parent_comment),
      post,
    } => {
      let content = markdown_to_html(&comment.content);
      (
        lang.notification_comment_reply_subject(&person.name),
        lang.notification_comment_reply_body(
          link,
          &content,
          &inbox_link,
          &parent_comment.content,
          &post.name,
          &person.name,
        ),
      )
    }
    NotificationEmailData::Reply {
      comment,
      person,
      parent_comment: None,
      post,
    } => {
      let content = markdown_to_html(&comment.content);
      (
        lang.notification_post_reply_subject(&person.name),
        lang.notification_post_reply_body(link, &content, &inbox_link, &post.name, &person.name),
      )
    }
    NotificationEmailData::PrivateMessage { sender, content } => {
      let sender_name = &sender.name;
      let content = markdown_to_html(content);
      (
        lang.notification_private_message_subject(sender_name),
        lang.notification_private_message_body(inbox_link, &content, sender_name),
      )
    }
    NotificationEmailData::ModAction {
      kind,
      reason,
      is_revert,
    } => {
      // Some actions like AdminAdd and ModAddToCommunity dont have any reason
      let reason = reason.unwrap_or_default();
      if is_revert {
        (
          lang.notification_mod_action_subject(kind).to_string(),
          lang.notification_mod_action_body(reason, inbox_link),
        )
      } else {
        (
          lang
            .notification_mod_action_reverted_subject(kind)
            .to_string(),
          lang.notification_mod_action_reverted_body(reason, inbox_link),
        )
      }
    }
  };

  if let Some(user_email) = local_user_view.local_user.email {
    send_email(
      subject,
      user_email,
      local_user_view.person.name,
      body,
      settings,
    );
  }
}
