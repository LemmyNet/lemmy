use crate::{
  http::{create_apub_response, create_apub_tombstone_response},
  objects::comment::ApubComment,
};
use actix_web::{body::AnyBody, web, web::Path, HttpResponse};
use diesel::result::Error::NotFound;
use lemmy_api_common::blocking;
use lemmy_apub_lib::traits::ApubObject;
use lemmy_db_schema::{newtypes::CommentId, source::comment::Comment, traits::Crud};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CommentQuery {
  comment_id: String,
}

/// Return the ActivityPub json representation of a local comment over HTTP.
#[tracing::instrument(skip(info, context))]
pub(crate) async fn get_apub_comment(
  info: Path<CommentQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<AnyBody>, LemmyError> {
  let id = CommentId(info.comment_id.parse::<i32>()?);
  let comment: ApubComment = blocking(context.pool(), move |conn| Comment::read(conn, id))
    .await??
    .into();
  if !comment.local {
    return Err(NotFound.into());
  }

  if !comment.deleted {
    Ok(create_apub_response(&comment.into_apub(&**context).await?))
  } else {
    Ok(create_apub_tombstone_response(&comment.to_tombstone()?))
  }
}
