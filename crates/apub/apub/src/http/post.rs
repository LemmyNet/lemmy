use super::check_community_content_fetchable;
use crate::protocol::collections::url_collection::UrlCollection;
use activitypub_federation::{config::Data, traits::Object};
use actix_web::{HttpRequest, HttpResponse, web};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{objects::post::ApubPost, utils::functions::context_url};
use lemmy_db_schema::{
  newtypes::PostId,
  source::{community::Community, post::Post},
  traits::Crud,
};
use lemmy_utils::{
  FEDERATION_CONTEXT,
  error::{LemmyErrorType, LemmyResult},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct PostQuery {
  post_id: String,
}

async fn get_post(
  info: web::Path<PostQuery>,
  context: &Data<LemmyContext>,
  request: &HttpRequest,
) -> LemmyResult<ApubPost> {
  let id = PostId(info.post_id.parse::<i32>()?);
  // Can't use PostView here because it excludes deleted/removed/local-only items
  let post: ApubPost = Post::read(&mut context.pool(), id).await?.into();
  let community = Community::read(&mut context.pool(), post.community_id).await?;

  check_community_content_fetchable(&community, request, context).await?;

  Ok(post)
}

/// Return the ActivityPub json representation of a local post over HTTP.
pub(crate) async fn get_apub_post(
  info: web::Path<PostQuery>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let post = get_post(info, &context, &request).await?;
  post.http_response(&FEDERATION_CONTEXT, &context).await
}

pub(crate) async fn get_apub_post_context(
  info: web::Path<PostQuery>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let post = get_post(info, &context, &request).await?;
  if !post.local {
    return Err(LemmyErrorType::NotFound.into());
  }
  UrlCollection::new_response(&post, context_url(&post.ap_id), &context).await
}
