use super::check_community_content_fetchable;
use crate::protocol::collections::url_collection::UrlCollection;
use activitypub_federation::{config::Data, traits::Object};
use actix_web::{HttpRequest, HttpResponse, web::Path};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{objects::comment::ApubComment, utils::functions::context_url};
use lemmy_db_schema::{
  newtypes::CommentId,
  source::{comment::Comment, community::Community, post::Post},
};
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::{
  FEDERATION_CONTEXT,
  error::{LemmyErrorType, LemmyResult},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CommentQuery {
  comment_id: String,
}

async fn get_comment(
  info: Path<CommentQuery>,
  context: &Data<LemmyContext>,
  request: &HttpRequest,
) -> LemmyResult<ApubComment> {
  let id = CommentId(info.comment_id.parse::<i32>()?);
  // Can't use CommentView here because it excludes deleted/removed/local-only items
  let comment: ApubComment = Comment::read(&mut context.pool(), id).await?.into();
  let post = Post::read(&mut context.pool(), comment.post_id).await?;
  let community = Community::read(&mut context.pool(), post.community_id).await?;
  check_community_content_fetchable(&community, request, context).await?;
  Ok(comment)
}

/// Return the ActivityPub json representation of a local comment over HTTP.
pub(crate) async fn get_apub_comment(
  info: Path<CommentQuery>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let comment = get_comment(info, &context, &request).await?;
  comment.http_response(&FEDERATION_CONTEXT, &context).await
}

pub(crate) async fn get_apub_comment_context(
  info: Path<CommentQuery>,
  context: Data<LemmyContext>,
  request: HttpRequest,
) -> LemmyResult<HttpResponse> {
  let comment = get_comment(info, &context, &request).await?;
  if !comment.local {
    return Err(LemmyErrorType::NotFound.into());
  }
  let post = Post::read(&mut context.pool(), comment.post_id).await?;
  UrlCollection::new_response(&post, context_url(&comment.ap_id), &context).await
}
