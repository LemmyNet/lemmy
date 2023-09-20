use actix_web::web::{Data, Json};
use lemmy_api_common::{
  comment::SaveComment,
  context::LemmyContext,
  utils::local_user_view_from_jwt,
  SuccessResponse,
};
use lemmy_db_schema::{
  source::comment::{CommentSaved, CommentSavedForm},
  traits::Saveable,
};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn save_comment(
  data: Json<SaveComment>,
  context: Data<LemmyContext>,
) -> Result<Json<SuccessResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

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

  Ok(Json(Default::default()))
}
