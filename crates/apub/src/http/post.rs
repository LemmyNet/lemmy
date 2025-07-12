use super::check_community_content_fetchable;
use activitypub_federation::{config::Data, traits::Object};
use actix_web::{web, HttpRequest, HttpResponse};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::objects::post::ApubPost;
use lemmy_db_schema::{
  newtypes::PostId,
  source::{community::Community, post::Post},
  traits::Crud,
};
use lemmy_utils::{error::LemmyResult, FEDERATION_CONTEXT};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct PostQuery {
  post_id: String,
}

/// Return the ActivityPub json representation of a local post over HTTP.
pub(crate) async fn get_apub_post(
  info: web::Path<PostQuery>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let id = PostId(info.post_id.parse::<i32>()?);
  // Can't use PostView here because it excludes deleted/removed/local-only items
  let post: ApubPost = Post::read(&mut context.pool(), id).await?.into();
  let community = Community::read(&mut context.pool(), post.community_id).await?;

  check_community_content_fetchable(&community, &request, &context).await?;

  post.http_response(&FEDERATION_CONTEXT, &context).await
}
