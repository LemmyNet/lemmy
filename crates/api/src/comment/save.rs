use actix_web::web::{Data, Json};
use lemmy_api_common::{
  comment::{CommentResponse, SaveComment},
  context::LemmyContext,
};
use lemmy_db_schema::{
  source::comment::{CommentActions, CommentSavedForm},
  traits::Saveable,
};
use lemmy_db_views::structs::{CommentView, LocalUserView};
use lemmy_utils::error::LemmyResult;

pub async fn save_comment(
  data: Json<SaveComment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommentResponse>> {
  let comment_saved_form = CommentSavedForm::new(local_user_view.person.id, data.comment_id);

  if data.save {
    CommentActions::save(&mut context.pool(), &comment_saved_form).await?;
  } else {
    CommentActions::unsave(&mut context.pool(), &comment_saved_form).await?;
  }

  let comment_id = data.comment_id;
  let comment_view = CommentView::read(
    &mut context.pool(),
    comment_id,
    Some(&local_user_view.local_user),
  )
  .await?;

  Ok(Json(CommentResponse {
    comment_view,
    recipient_ids: Vec::new(),
  }))
}
