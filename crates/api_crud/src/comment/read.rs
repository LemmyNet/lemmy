use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  build_response::build_comment_response,
  comment::{CommentResponse, GetComment},
  context::LemmyContext,
  utils::{check_private_instance, local_user_view_from_jwt_opt},
};
use lemmy_db_schema::source::local_site::LocalSite;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn get_comment(
  data: Query<GetComment>,
  context: Data<LemmyContext>,
) -> Result<Json<CommentResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt_opt(data.auth.as_ref(), &context).await;
  let local_site = LocalSite::read(&mut context.pool()).await?;

  check_private_instance(&local_user_view, &local_site)?;

  Ok(Json(
    build_comment_response(&context, data.id, local_user_view, vec![]).await?,
  ))
}
