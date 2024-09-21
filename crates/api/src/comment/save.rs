use actix_web::web::{Data, Json};
use lemmy_api_common::{
  comment::{CommentResponse, SaveComment},
  context::LemmyContext,
};
use lemmy_db_schema::{
  source::comment::{CommentSaved, CommentSavedForm},
  traits::Saveable,
};
use lemmy_db_views::structs::{CommentView, LocalUserView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn save_comment(
  data: Json<SaveComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponse>> {
  let comment_saved_form = CommentSavedForm {
    comment_id: data.comment_id,
    person_id: local_user_view.person.id,
  };

  if data.save {
    CommentSaved::save(&mut context.pool(), &comment_saved_form)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntSaveComment)?;
  } else {
    CommentSaved::unsave(&mut context.pool(), &comment_saved_form)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntSaveComment)?;
  }

  let comment_id = data.comment_id;
  let comment_view = CommentView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
  )
  .await?
  .ok_or(LemmyErrorType::CouldntFindComment)?;

  Ok(Json(CommentResponse {
    comment_view,
    recipient_ids: Vec::new(),
  }))
}
