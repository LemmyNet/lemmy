use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  comment::{CommentResponse, SaveComment},
  utils::{blocking, get_local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::comment::{CommentSaved, CommentSavedForm},
  traits::Saveable,
};
use lemmy_db_views::structs::CommentView;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for SaveComment {
  type Response = CommentResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<CommentResponse, LemmyError> {
    let data: &SaveComment = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let comment_saved_form = CommentSavedForm {
      comment_id: data.comment_id,
      person_id: local_user_view.person.id,
    };

    if data.save {
      let save_comment = move |conn: &mut _| CommentSaved::save(conn, &comment_saved_form);
      blocking(context.pool(), save_comment)
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_save_comment"))?;
    } else {
      let unsave_comment = move |conn: &mut _| CommentSaved::unsave(conn, &comment_saved_form);
      blocking(context.pool(), unsave_comment)
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_save_comment"))?;
    }

    let comment_id = data.comment_id;
    let person_id = local_user_view.person.id;
    let comment_view = blocking(context.pool(), move |conn| {
      CommentView::read(conn, comment_id, Some(person_id))
    })
    .await??;

    Ok(CommentResponse {
      comment_view,
      recipient_ids: Vec::new(),
      form_id: None,
    })
  }
}
