use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  post::{PostResponse, SavePost},
  utils::{local_user_view_from_jwt, mark_post_as_read},
};
use lemmy_db_schema::{
  source::post::{PostSaved, PostSavedForm},
  traits::Saveable,
};
use lemmy_db_views::structs::PostView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[async_trait::async_trait(?Send)]
impl Perform for SavePost {
  type Response = PostResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<PostResponse, LemmyError> {
    let data: &SavePost = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let post_saved_form = PostSavedForm {
      post_id: data.post_id,
      person_id: local_user_view.person.id,
    };

    if data.save {
      PostSaved::save(&mut context.pool(), &post_saved_form)
        .await
        .with_lemmy_type(LemmyErrorType::CouldntSavePost)?;
    } else {
      PostSaved::unsave(&mut context.pool(), &post_saved_form)
        .await
        .with_lemmy_type(LemmyErrorType::CouldntSavePost)?;
    }

    let post_id = data.post_id;
    let person_id = local_user_view.person.id;
    let post_view = PostView::read(&mut context.pool(), post_id, Some(person_id), false).await?;

    // Mark the post as read
    mark_post_as_read(person_id, post_id, &mut context.pool()).await?;

    Ok(PostResponse { post_view })
  }
}
