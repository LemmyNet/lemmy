pub mod comment;
pub mod community;
pub mod post;
pub mod site;
pub mod user;
pub mod websocket;

use diesel::PgConnection;
use lemmy_db::{
  comment::Comment,
  post::Post,
  user::User_,
  user_mention::{UserMention, UserMentionForm},
  Crud,
  DbPool,
};
use lemmy_utils::{email::send_email, settings::Settings, utils::MentionData, LemmyError};
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WebFingerLink {
  pub rel: Option<String>,
  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub type_: Option<String>,
  pub href: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub template: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebFingerResponse {
  pub subject: String,
  pub aliases: Vec<String>,
  pub links: Vec<WebFingerLink>,
}

pub async fn blocking<F, T>(pool: &DbPool, f: F) -> Result<T, LemmyError>
where
  F: FnOnce(&diesel::PgConnection) -> T + Send + 'static,
  T: Send + 'static,
{
  let pool = pool.clone();
  let res = actix_web::web::block(move || {
    let conn = pool.get()?;
    let res = (f)(&conn);
    Ok(res) as Result<_, LemmyError>
  })
  .await?;

  Ok(res)
}

pub async fn send_local_notifs(
  mentions: Vec<MentionData>,
  comment: Comment,
  user: &User_,
  post: Post,
  pool: &DbPool,
  do_send_email: bool,
) -> Result<Vec<i32>, LemmyError> {
  let user2 = user.clone();
  let ids = blocking(pool, move |conn| {
    do_send_local_notifs(conn, &mentions, &comment, &user2, &post, do_send_email)
  })
  .await?;

  Ok(ids)
}

fn do_send_local_notifs(
  conn: &PgConnection,
  mentions: &[MentionData],
  comment: &Comment,
  user: &User_,
  post: &Post,
  do_send_email: bool,
) -> Vec<i32> {
  let mut recipient_ids = Vec::new();
  let hostname = &Settings::get().get_protocol_and_hostname();

  // Send the local mentions
  for mention in mentions
    .iter()
    .filter(|m| m.is_local() && m.name.ne(&user.name))
    .collect::<Vec<&MentionData>>()
  {
    if let Ok(mention_user) = User_::read_from_name(&conn, &mention.name) {
      // TODO
      // At some point, make it so you can't tag the parent creator either
      // This can cause two notifications, one for reply and the other for mention
      recipient_ids.push(mention_user.id);

      let user_mention_form = UserMentionForm {
        recipient_id: mention_user.id,
        comment_id: comment.id,
        read: None,
      };

      // Allow this to fail softly, since comment edits might re-update or replace it
      // Let the uniqueness handle this fail
      match UserMention::create(&conn, &user_mention_form) {
        Ok(_mention) => (),
        Err(_e) => error!("{}", &_e),
      };

      // Send an email to those users that have notifications on
      if do_send_email && mention_user.send_notifications_to_email {
        if let Some(mention_email) = mention_user.email {
          let subject = &format!("{} - Mentioned by {}", Settings::get().hostname, user.name,);
          let html = &format!(
            "<h1>User Mention</h1><br><div>{} - {}</div><br><a href={}/inbox>inbox</a>",
            user.name, comment.content, hostname
          );
          match send_email(subject, &mention_email, &mention_user.name, html) {
            Ok(_o) => _o,
            Err(e) => error!("{}", e),
          };
        }
      }
    }
  }

  // Send notifs to the parent commenter / poster
  match comment.parent_id {
    Some(parent_id) => {
      if let Ok(parent_comment) = Comment::read(&conn, parent_id) {
        if parent_comment.creator_id != user.id {
          if let Ok(parent_user) = User_::read(&conn, parent_comment.creator_id) {
            recipient_ids.push(parent_user.id);

            if do_send_email && parent_user.send_notifications_to_email {
              if let Some(comment_reply_email) = parent_user.email {
                let subject = &format!("{} - Reply from {}", Settings::get().hostname, user.name,);
                let html = &format!(
                  "<h1>Comment Reply</h1><br><div>{} - {}</div><br><a href={}/inbox>inbox</a>",
                  user.name, comment.content, hostname
                );
                match send_email(subject, &comment_reply_email, &parent_user.name, html) {
                  Ok(_o) => _o,
                  Err(e) => error!("{}", e),
                };
              }
            }
          }
        }
      }
    }
    // Its a post
    None => {
      if post.creator_id != user.id {
        if let Ok(parent_user) = User_::read(&conn, post.creator_id) {
          recipient_ids.push(parent_user.id);

          if do_send_email && parent_user.send_notifications_to_email {
            if let Some(post_reply_email) = parent_user.email {
              let subject = &format!("{} - Reply from {}", Settings::get().hostname, user.name,);
              let html = &format!(
                "<h1>Post Reply</h1><br><div>{} - {}</div><br><a href={}/inbox>inbox</a>",
                user.name, comment.content, hostname
              );
              match send_email(subject, &post_reply_email, &parent_user.name, html) {
                Ok(_o) => _o,
                Err(e) => error!("{}", e),
              };
            }
          }
        }
      }
    }
  };
  recipient_ids
}
