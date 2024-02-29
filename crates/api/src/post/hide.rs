use actix_web::web::{Data, Json};
use lemmy_api_common::{context::LemmyContext, post::HidePost, SuccessResponse};
use lemmy_db_schema::source::post::PostHide;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType, MAX_API_PARAM_ELEMENTS};
use std::collections::HashSet;

#[tracing::instrument(skip(context))]
pub async fn hide_post(
  data: Json<HidePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SuccessResponse>, LemmyError> {
  let post_ids = HashSet::from_iter(data.post_ids.clone());

  if post_ids.len() > MAX_API_PARAM_ELEMENTS {
    Err(LemmyErrorType::TooManyItems)?;
  }

  let person_id = local_user_view.person.id;

  // Mark the post as hidden / unhidden
  if data.hide {
    PostHide::hide(&mut context.pool(), post_ids, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntHidePost)?;
  } else {
    PostHide::unhide(&mut context.pool(), post_ids, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntHidePost)?;
  }

  Ok(Json(SuccessResponse::default()))
}
