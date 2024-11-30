use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  post::{HidePost, PostResponse},
};
use lemmy_db_schema::source::post::PostHide;
use lemmy_db_views::structs::{LocalUserView, PostView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn hide_post(
  data: Json<HidePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let person_id = local_user_view.person.id;
  let post_id = data.post_id;

  // Mark the post as hidden / unhidden
  if data.hide {
    PostHide::hide(&mut context.pool(), post_id, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntHidePost)?;
  } else {
    PostHide::unhide(&mut context.pool(), post_id, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntHidePost)?;
  }

  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    false,
  )
  .await?;

  Ok(Json(PostResponse { post_view }))
}
