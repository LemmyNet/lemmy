use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  post::{PostResponse, SavePost},
  utils::{blocking, get_local_user_view_from_jwt, mark_post_as_read},
};
use lemmy_db_schema::{
  source::post::{PostSaved, PostSavedForm},
  traits::Saveable,
};
use lemmy_db_views::structs::PostView;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for SavePost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &SavePost = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let post_saved_form = PostSavedForm {
      post_id: data.post_id,
      person_id: local_user_view.person.id,
    };

    if data.save {
      let save = move |conn: &mut _| PostSaved::save(conn, &post_saved_form);
      blocking(context.pool(), save)
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_save_post"))?;
    } else {
      let unsave = move |conn: &mut _| PostSaved::unsave(conn, &post_saved_form);
      blocking(context.pool(), unsave)
        .await?
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_save_post"))?;
    }

    let post_id = data.post_id;
    let person_id = local_user_view.person.id;
    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, Some(person_id))
    })
    .await??;

    // Mark the post as read
    mark_post_as_read(person_id, post_id, context.pool()).await?;

    Ok(PostResponse { post_view })
  }
}
