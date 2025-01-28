use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  post::{PostResponse, SavePost},
};
use lemmy_db_schema::{
  source::post::{PostRead, PostReadForm, PostSaved, PostSavedForm},
  traits::Saveable,
};
use lemmy_db_views::structs::{LocalUserView, PostView};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

pub async fn save_post(
  data: Json<SavePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let post_saved_form = PostSavedForm::new(data.post_id, local_user_view.person.id);

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
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    false,
  )
  .await?;

  let read_form = PostReadForm::new(post_id, person_id);
  PostRead::mark_as_read(&mut context.pool(), &read_form).await?;

  Ok(Json(PostResponse { post_view }))
}
