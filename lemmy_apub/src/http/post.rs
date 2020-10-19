use crate::{
  http::{create_apub_response, create_apub_tombstone_response},
  ToApub,
};
use actix_web::{body::Body, web, HttpResponse};
use diesel::result::Error::NotFound;
use lemmy_db::post::Post;
use lemmy_structs::blocking;
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PostQuery {
  post_id: String,
}

/// Return the ActivityPub json representation of a local post over HTTP.
pub async fn get_apub_post(
  info: web::Path<PostQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse<Body>, LemmyError> {
  let id = info.post_id.parse::<i32>()?;
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
