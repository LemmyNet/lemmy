use crate::{
  http::{create_apub_response, create_apub_tombstone_response},
  objects::post::ApubPost,
};
use activitypub_federation::traits::ApubObject;
use actix_web::{web, HttpResponse};
use diesel::result::Error::NotFound;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{newtypes::PostId, source::post::Post, traits::Crud};
use lemmy_utils::error::LemmyError;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct PostQuery {
  post_id: String,
}

/// Return the ActivityPub json representation of a local post over HTTP.
#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_post(
  info: web::Path<PostQuery>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let id = PostId(info.post_id.parse::<i32>()?);
  let post: ApubPost = Post::read(context.pool(), id).await?.into();
  if !post.local {
    return Err(NotFound.into());
  }

  if !post.deleted && !post.removed {
    Ok(create_apub_response(&post.into_apub(&context).await?))
  } else {
    Ok(create_apub_tombstone_response(post.ap_id.clone()))
  }
}
