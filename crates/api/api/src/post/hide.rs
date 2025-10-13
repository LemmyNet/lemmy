use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::source::post::{PostActions, PostHideForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::{
  api::{HidePost, PostResponse},
  PostView,
};
use lemmy_utils::error::LemmyResult;

pub async fn hide_post(
  data: Json<HidePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  check_local_user_valid(&local_user_view)?;
  let person_id = local_user_view.person.id;
  let local_instance_id = local_user_view.person.instance_id;
  let post_id = data.post_id;

  let hide_form = PostHideForm::new(post_id, person_id);

  // Mark the post as hidden / unhidden
  if data.hide {
    PostActions::hide(&mut context.pool(), &hide_form).await?;
  } else {
    PostActions::unhide(&mut context.pool(), &hide_form).await?;
  }

  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    local_instance_id,
    false,
  )
  .await?;

  Ok(Json(PostResponse { post_view }))
}
