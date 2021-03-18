pub mod comment;
pub mod community;
pub mod person;
pub mod post;
pub mod site;
pub mod websocket;

use diesel::PgConnection;
use lemmy_db_queries::{Crud, DbPool};
use lemmy_db_schema::source::{
  comment::Comment,
  person::Person,
  person_mention::{PersonMention, PersonMentionForm},
  post::Post,
};
use lemmy_db_views::local_user_view::LocalUserView;
use lemmy_utils::{email::send_email, settings::structs::Settings, utils::MentionData, LemmyError};
use log::error;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct WebFingerLink {
  pub rel: Option<String>,
  #[serde(rename(serialize = "type", deserialize = "type"))]
  pub type_: Option<String>,
  pub href: Option<Url>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub template: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebFingerResponse {
  pub subject: String,
  pub aliases: Vec<Url>,
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
  person: Person,
  post: Post,
  pool: &DbPool,
  do_send_email: bool,
) -> Result<Vec<i32>, LemmyError> {
  let ids = blocking(pool, move |conn| {
    do_send_local_notifs(conn, &mentions, &comment, &person, &post, do_send_email)
  })
  .await?;

  Ok(ids)
}

fn do_send_local_notifs(
  conn: &PgConnection,
  mentions: &[MentionData],
  comment: &Comment,
  person: &Person,
  post: &Post,
  do_send_email: bool,
) -> Vec<i32> {
  let mut recipient_ids = Vec::new();

  // Send the local mentions
  for mention in mentions
    .iter()
    .filter(|m| m.is_local() && m.name.ne(&person.name))
    .collect::<Vec<&MentionData>>()
  {
    if let Ok(mention_user_view) = LocalUserView::read_from_name(&conn, &mention.name) {
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
      PersonMention::create(&conn, &user_mention_form).ok();

      // Send an email to those local users that have notifications on
      if do_send_email {
        send_email_to_user(
          &mention_user_view,
          "Mentioned by",
          "Person Mention",
          &comment.content,
        )
      }
    }
  }

  // Send notifs to the parent commenter / poster
  match comment.parent_id {
    Some(parent_id) => {
      if let Ok(parent_comment) = Comment::read(&conn, parent_id) {
        if parent_comment.creator_id != person.id {
          if let Ok(parent_user_view) = LocalUserView::read_person(&conn, parent_comment.creator_id)
          {
            recipient_ids.push(parent_user_view.local_user.id);

            if do_send_email {
              send_email_to_user(
                &parent_user_view,
                "Reply from",
                "Comment Reply",
                &comment.content,
              )
            }
          }
        }
      }
    }
    // Its a post
    None => {
      if post.creator_id != person.id {
        if let Ok(parent_user_view) = LocalUserView::read_person(&conn, post.creator_id) {
          recipient_ids.push(parent_user_view.local_user.id);

          if do_send_email {
            send_email_to_user(
              &parent_user_view,
              "Reply from",
              "Post Reply",
              &comment.content,
            )
          }
        }
      }
    }
  };
  recipient_ids
}

pub fn send_email_to_user(
  local_user_view: &LocalUserView,
  subject_text: &str,
  body_text: &str,
  comment_content: &str,
) {
  if local_user_view.person.banned || !local_user_view.local_user.send_notifications_to_email {
    return;
  }

  if let Some(user_email) = &local_user_view.local_user.email {
    let subject = &format!(
      "{} - {} {}",
      subject_text,
      Settings::get().hostname(),
      local_user_view.person.name,
    );
    let html = &format!(
      "<h1>{}</h1><br><div>{} - {}</div><br><a href={}/inbox>inbox</a>",
      body_text,
      local_user_view.person.name,
      comment_content,
      Settings::get().get_protocol_and_hostname()
    );
    match send_email(subject, &user_email, &local_user_view.person.name, html) {
      Ok(_o) => _o,
      Err(e) => error!("{}", e),
    };
  }
}
