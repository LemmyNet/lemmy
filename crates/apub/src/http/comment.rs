use crate::{
  http::{create_apub_response, create_apub_tombstone_response, redirect_remote_object},
  objects::comment::ApubComment,
};
use activitypub_federation::{config::Data, traits::Object};
use actix_web::{web::Path, HttpResponse};
use lemmy_api_common::{context::LemmyContext, utils::check_community_valid};
use lemmy_db_schema::newtypes::CommentId;
use lemmy_db_views::structs::CommentView;
use lemmy_utils::error::LemmyError;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CommentQuery {
  comment_id: String,
}

/// Return the ActivityPub json representation of a local comment over HTTP.
#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_comment(
  info: Path<CommentQuery>,
  context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let id = CommentId(info.comment_id.parse::<i32>()?);
  let comment_view = CommentView::read(&mut context.pool(), id, None).await?;
  check_community_valid(&comment_view.community)?;

  let comment: ApubComment = comment_view.comment.into();
  if !comment.local {
    Ok(redirect_remote_object(&comment.ap_id))
  } else if !comment.deleted && !comment.removed {
    create_apub_response(&comment.into_json(&context).await?)
  } else {
    create_apub_tombstone_response(comment.ap_id.clone())
  }
}
