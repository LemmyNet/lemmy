use actix_web::web::{Data, Json};
use lemmy_api_common::{context::LemmyContext, post::MarkPostAsRead, SuccessResponse};
use lemmy_db_schema::source::post::PostRead;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult, MAX_API_PARAM_ELEMENTS};
use std::collections::HashSet;

#[tracing::instrument(skip(context))]
pub async fn mark_post_as_read(
  data: Json<MarkPostAsRead>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let post_ids = HashSet::from_iter(data.post_ids.clone());

  if post_ids.len() > MAX_API_PARAM_ELEMENTS {
    Err(LemmyErrorType::TooManyItems)?;
  }

  let person_id = local_user_view.person.id;

  // Mark the post as read / unread
  if data.read {
    PostRead::mark_as_read(&mut context.pool(), post_ids, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntMarkPostAsRead)?;
  } else {
    PostRead::mark_as_unread(&mut context.pool(), post_ids, person_id)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntMarkPostAsRead)?;
  }

  Ok(Json(SuccessResponse::default()))
}
