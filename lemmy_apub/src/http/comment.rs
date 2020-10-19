use crate::{
  http::{create_apub_response, create_apub_tombstone_response},
  ToApub,
};
use actix_web::{body::Body, web, web::Path, HttpResponse};
use diesel::result::Error::NotFound;
use lemmy_db::{comment::Comment, Crud};
use lemmy_structs::blocking;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CommentQuery {
  comment_id: String,
}

/// Return the ActivityPub json representation of a local comment over HTTP.
pub async fn get_apub_comment(
  info: Path<CommentQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let id = info.comment_id.parse::<i32>()?;
  let comment = blocking(context.pool(), move |conn| Comment::read(conn, id)).await??;
  if !comment.local {
    return Err(NotFound.into());
  }

  if !comment.deleted {
    Ok(create_apub_response(
      &comment.to_apub(context.pool()).await?,
    ))
  } else {
    Ok(create_apub_tombstone_response(&comment.to_tombstone()?))
  }
}
