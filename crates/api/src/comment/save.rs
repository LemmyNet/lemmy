use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{CommentResponse, SaveComment},
  context::LemmyContext,
  utils::local_user_view_from_jwt,
};
use lemmy_db_schema::{
  source::comment::{CommentSaved, CommentSavedForm},
  traits::Saveable,
};
use lemmy_db_views::structs::CommentView;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for SaveComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CommentResponse, LemmyError> {
    let data: &SaveComment = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let comment_saved_form = CommentSavedForm {
      comment_id: data.comment_id,
      person_id: local_user_view.person.id,
    };

    if data.save {
      CommentSaved::save(context.pool(), &comment_saved_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_save_comment"))?;
    } else {
      CommentSaved::unsave(context.pool(), &comment_saved_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_save_comment"))?;
    }

    let comment_id = data.comment_id;
    let person_id = local_user_view.person.id;
    let comment_view = CommentView::read(context.pool(), comment_id, Some(person_id)).await?;

    Ok(CommentResponse {
      comment_view,
      recipient_ids: Vec::new(),
      form_id: None,
    })
  }
}
