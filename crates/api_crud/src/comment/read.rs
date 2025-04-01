use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  build_response::build_comment_response,
  comment::{CommentResponse, GetComment},
  context::LemmyContext,
  utils::check_private_instance,
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::error::LemmyResult;

pub async fn get_comment(
  data: Query<GetComment>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<CommentResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;

  check_private_instance(&local_user_view, &local_site)?;

  Ok(Json(
    build_comment_response(&context, data.id, local_user_view, vec![]).await?,
  ))
}
