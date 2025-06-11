use crate::{inbox_link, send_email, user_language};
use lemmy_db_schema::{
  newtypes::DbUrl,
  source::{comment::Comment, person::Person, post::Post},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{
  error::LemmyResult,
  settings::structs::Settings,
  utils::markdown::markdown_to_html,
};
use tracing::warn;

pub async fn send_mention_email(
  mention_user_view: &LocalUserView,
  content: &str,
  person: &Person,
  link: DbUrl,
  settings: &Settings,
) {
  let inbox_link = inbox_link(settings);
  let lang = user_language(mention_user_view);
  let content = markdown_to_html(content);
  send_email_to_user(
    mention_user_view,
    &lang.notification_mentioned_by_subject(&person.name),
    &lang.notification_mentioned_by_body(&link, &content, &inbox_link, &person.name),
    settings,
  )
  .await
}

pub async fn send_comment_reply_email(
  parent_user_view: &LocalUserView,
  comment: &Comment,
  person: &Person,
  parent_comment: &Comment,
  post: &Post,
  settings: &Settings,
) -> LemmyResult<()> {
  let inbox_link = inbox_link(settings);
  let lang = user_language(parent_user_view);
  let content = markdown_to_html(&comment.content);
  send_email_to_user(
    parent_user_view,
    &lang.notification_comment_reply_subject(&person.name),
    &lang.notification_comment_reply_body(
      comment.local_url(settings)?,
      &content,
      &inbox_link,
      &parent_comment.content,
      &post.name,
      &person.name,
    ),
    settings,
  )
  .await;
  Ok(())
}

pub async fn send_post_reply_email(
  parent_user_view: &LocalUserView,
  comment: &Comment,
  person: &Person,
  post: &Post,
  settings: &Settings,
) -> LemmyResult<()> {
  let inbox_link = inbox_link(settings);
  let lang = user_language(parent_user_view);
  let content = markdown_to_html(&comment.content);
  send_email_to_user(
    parent_user_view,
    &lang.notification_post_reply_subject(&person.name),
    &lang.notification_post_reply_body(
      comment.local_url(settings)?,
      &content,
      &inbox_link,
      &post.name,
      &person.name,
    ),
    settings,
  )
  .await;
  Ok(())
}

pub async fn send_private_message_email(
  sender: &LocalUserView,
  local_recipient: &LocalUserView,
  content: &str,
  settings: &Settings,
) {
  let inbox_link = inbox_link(settings);
  let lang = user_language(local_recipient);
  let sender_name = &sender.person.name;
  let content = markdown_to_html(content);
  send_email_to_user(
    local_recipient,
    &lang.notification_private_message_subject(sender_name),
    &lang.notification_private_message_body(inbox_link, &content, sender_name),
    settings,
  )
  .await;
}

async fn send_email_to_user(
  local_user_view: &LocalUserView,
  subject: &str,
  body: &str,
  settings: &Settings,
) {
  let banned = local_user_view
    .instance_actions
    .as_ref()
    .and_then(|i| i.received_ban_at)
    .is_some();
  if banned || !local_user_view.local_user.send_notifications_to_email {
    return;
  }

  if let Some(user_email) = &local_user_view.local_user.email {
    match send_email(
      subject,
      user_email,
      &local_user_view.person.name,
      body,
      settings,
    )
    .await
    {
      Ok(_o) => _o,
      Err(e) => warn!("{}", e),
    };
  }
}
