use crate::{fetcher::fetch::fetch_remote_object, objects::FromApub, NoteExt, PageExt};
use anyhow::anyhow;
use diesel::result::Error::NotFound;
use lemmy_db_queries::{ApubObject, Crud};
use lemmy_db_schema::source::{comment::Comment, post::Post};
use lemmy_structs::blocking;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use log::debug;
use url::Url;

/// Gets a post by its apub ID. If it exists locally, it is returned directly. Otherwise it is
/// pulled from its apub ID, inserted and returned.
///
/// The parent community is also pulled if necessary. Comments are not pulled.
pub(crate) async fn get_or_fetch_and_insert_post(
  post_ap_id: &Url,
  context: &LemmyContext,
  recursion_counter: &mut i32,
) -> Result<Post, LemmyError> {
  let post_ap_id_owned = post_ap_id.to_owned();
  let post = blocking(context.pool(), move |conn| {
    Post::read_from_apub_id(conn, post_ap_id_owned.as_str())
  })
  .await?;

  match post {
    Ok(p) => Ok(p),
    Err(NotFound {}) => {
      debug!("Fetching and creating remote post: {}", post_ap_id);
      let page =
        fetch_remote_object::<PageExt>(context.client(), post_ap_id, recursion_counter).await?;
      let post = Post::from_apub(&page, context, post_ap_id.to_owned(), recursion_counter).await?;

      Ok(post)
    }
    Err(e) => Err(e.into()),
  }
}

/// Gets a comment by its apub ID. If it exists locally, it is returned directly. Otherwise it is
/// pulled from its apub ID, inserted and returned.
///
/// The parent community, post and comment are also pulled if necessary.
pub(crate) async fn get_or_fetch_and_insert_comment(
  comment_ap_id: &Url,
  context: &LemmyContext,
  recursion_counter: &mut i32,
) -> Result<Comment, LemmyError> {
  let comment_ap_id_owned = comment_ap_id.to_owned();
  let comment = blocking(context.pool(), move |conn| {
    Comment::read_from_apub_id(conn, comment_ap_id_owned.as_str())
  })
  .await?;

  match comment {
    Ok(p) => Ok(p),
    Err(NotFound {}) => {
      debug!(
        "Fetching and creating remote comment and its parents: {}",
        comment_ap_id
      );
      let comment =
        fetch_remote_object::<NoteExt>(context.client(), comment_ap_id, recursion_counter).await?;
      let comment = Comment::from_apub(
        &comment,
        context,
        comment_ap_id.to_owned(),
        recursion_counter,
      )
      .await?;

      let post_id = comment.post_id;
      let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
      if post.locked {
        return Err(anyhow!("Post is locked").into());
      }

      Ok(comment)
    }
    Err(e) => Err(e.into()),
  }
}
