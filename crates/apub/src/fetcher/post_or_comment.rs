use crate::objects::{comment::Note, post::Page, FromApub};
use activitystreams::chrono::NaiveDateTime;
use diesel::{result::Error, PgConnection};
use lemmy_db_queries::ApubObject;
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentForm},
    post::{Post, PostForm},
  },
  DbUrl,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;
use url::Url;

#[derive(Clone, Debug)]
pub enum PostOrComment {
  Comment(Box<Comment>),
  Post(Box<Post>),
}

pub enum PostOrCommentForm {
  PostForm(PostForm),
  CommentForm(CommentForm),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum PageOrNote {
  Page(Page),
  Note(Note),
}

#[async_trait::async_trait(?Send)]
impl ApubObject for PostOrComment {
  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    None
  }

  // TODO: this can probably be implemented using a single sql query
  fn read_from_apub_id(conn: &PgConnection, object_id: &DbUrl) -> Result<Self, Error>
  where
    Self: Sized,
  {
    let post = Post::read_from_apub_id(conn, object_id);
    Ok(match post {
      Ok(o) => PostOrComment::Post(Box::new(o)),
      Err(_) => PostOrComment::Comment(Box::new(Comment::read_from_apub_id(conn, object_id)?)),
    })
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for PostOrComment {
  type ApubType = PageOrNote;

  async fn from_apub(
    apub: &PageOrNote,
    context: &LemmyContext,
    expected_domain: &Url,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError>
  where
    Self: Sized,
  {
    Ok(match apub {
      PageOrNote::Page(p) => PostOrComment::Post(Box::new(
        Post::from_apub(p, context, expected_domain, request_counter).await?,
      )),
      PageOrNote::Note(n) => PostOrComment::Comment(Box::new(
        Comment::from_apub(n, context, expected_domain, request_counter).await?,
      )),
    })
  }
}

impl PostOrComment {
  pub(crate) fn ap_id(&self) -> Url {
    match self {
      PostOrComment::Post(p) => p.ap_id.clone(),
      PostOrComment::Comment(c) => c.ap_id.clone(),
    }
    .into()
  }
}
