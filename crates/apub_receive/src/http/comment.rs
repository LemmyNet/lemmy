use crate::http::{create_apub_response, create_apub_tombstone_response};
use actix_web::{body::Body, web, web::Path, HttpResponse};
use diesel::result::Error::NotFound;
use lemmy_api_common::blocking;
use lemmy_apub::objects::ToApub;
use lemmy_db_queries::Crud;
use lemmy_db_schema::{source::comment::Comment, CommentId};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CommentQuery {
  comment_id: String,
}

/// Return the ActivityPub json representation of a local comment over HTTP.
pub(crate) async fn get_apub_comment(
  info: Path<CommentQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let id = CommentId(info.comment_id.parse::<i32>()?);
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
