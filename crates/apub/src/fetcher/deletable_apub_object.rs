use crate::fetcher::post_or_comment::PostOrComment;
use lemmy_api_common::utils::blocking;
use lemmy_db_queries::source::{
  comment::Comment_,
  community::Community_,
  person::Person_,
  post::Post_,
};
use lemmy_db_schema::source::{
  comment::Comment,
  community::Community,
  person::Person,
  post::Post,
  site::Site,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;

// TODO: merge this trait with ApubObject (means that db_schema needs to depend on apub_lib)
#[async_trait::async_trait(?Send)]
pub trait DeletableApubObject {
  // TODO: pass in tombstone with summary field, to decide between remove/delete
  async fn delete(self, context: &LemmyContext) -> Result<(), LemmyError>;
}

#[async_trait::async_trait(?Send)]
impl DeletableApubObject for Community {
  async fn delete(self, context: &LemmyContext) -> Result<(), LemmyError> {
    let id = self.id;
    blocking(context.pool(), move |conn| {
      Community::update_deleted(conn, id, true)
    })
    .await??;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl DeletableApubObject for Person {
  async fn delete(self, context: &LemmyContext) -> Result<(), LemmyError> {
    let id = self.id;
    blocking(context.pool(), move |conn| Person::delete_account(conn, id)).await??;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl DeletableApubObject for Post {
  async fn delete(self, context: &LemmyContext) -> Result<(), LemmyError> {
    let id = self.id;
    blocking(context.pool(), move |conn| {
      Post::update_deleted(conn, id, true)
    })
    .await??;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl DeletableApubObject for Comment {
  async fn delete(self, context: &LemmyContext) -> Result<(), LemmyError> {
    let id = self.id;
    blocking(context.pool(), move |conn| {
      Comment::update_deleted(conn, id, true)
    })
    .await??;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl DeletableApubObject for PostOrComment {
  async fn delete(self, context: &LemmyContext) -> Result<(), LemmyError> {
    match self {
      PostOrComment::Comment(c) => {
        blocking(context.pool(), move |conn| {
          Comment::update_deleted(conn, c.id, true)
        })
        .await??;
      }
      PostOrComment::Post(p) => {
        blocking(context.pool(), move |conn| {
          Post::update_deleted(conn, p.id, true)
        })
        .await??;
      }
    }

    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl DeletableApubObject for Site {
  async fn delete(self, _context: &LemmyContext) -> Result<(), LemmyError> {
    // not implemented, ignore
    Ok(())
  }
}
