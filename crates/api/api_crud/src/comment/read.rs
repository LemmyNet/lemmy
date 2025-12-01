use actix_web::web::{Data, Json, Query};
use lemmy_api_utils::{
  build_response::build_comment_response,
  context::LemmyContext,
  utils::check_private_instance,
};
use lemmy_db_views_comment::api::{CommentResponse, GetComment};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::LemmyResult;

pub async fn get_comment(
  Query(data): Query<GetComment>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<CommentResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = site_view.local_site;
  let local_instance_id = site_view.site.instance_id;

  check_private_instance(&local_user_view, &local_site)?;

  Ok(Json(
    build_comment_response(&context, data.id, local_user_view, local_instance_id).await?,
  ))
}
