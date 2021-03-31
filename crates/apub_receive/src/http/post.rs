use crate::http::{create_apub_response, create_apub_tombstone_response};
use actix_web::{body::Body, web, HttpResponse};
use diesel::result::Error::NotFound;
use lemmy_api_common::blocking;
use lemmy_apub::objects::ToApub;
use lemmy_db_queries::Crud;
use lemmy_db_schema::{source::post::Post, PostId};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct PostQuery {
  post_id: String,
}

/// Return the ActivityPub json representation of a local post over HTTP.
pub(crate) async fn get_apub_post(
  info: web::Path<PostQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let id = PostId(info.post_id.parse::<i32>()?);
  let post = blocking(context.pool(), move |conn| Post::read(conn, id)).await??;
  if !post.local {
    return Err(NotFound.into());
  }

  if !post.deleted {
    Ok(create_apub_response(&post.to_apub(context.pool()).await?))
  } else {
    Ok(create_apub_tombstone_response(&post.to_tombstone()?))
  }
}
