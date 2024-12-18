use actix_web::web::{Data, Json, Path};
use lemmy_api_common::{
  build_response::build_comment_response, comment::CommentResponse, context::LemmyContext,
  utils::check_private_instance,
};
use lemmy_db_schema::{newtypes::CommentId, source::local_site::LocalSite};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn get_comment(
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
  path: Path<CommentId>,
) -> LemmyResult<Json<CommentResponse>> {
  let id = path.into_inner();
  let local_site = LocalSite::read(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site)?;

  Ok(Json(
    build_comment_response(&context, id, local_user_view, vec![]).await?,
  ))
}
