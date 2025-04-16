use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  post::{PostResponse, SavePost},
};
use lemmy_db_schema::{
  source::post::{PostActions, PostReadForm, PostSavedForm},
  traits::{Readable, Saveable},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_post::PostView;
use lemmy_utils::error::LemmyResult;

pub async fn save_post(
  data: Json<SavePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PostResponse>> {
  let post_saved_form = PostSavedForm::new(data.post_id, local_user_view.person.id);

  if data.save {
    PostActions::save(&mut context.pool(), &post_saved_form).await?;
  } else {
    PostActions::unsave(&mut context.pool(), &post_saved_form).await?;
  }

  let post_id = data.post_id;
  let person_id = local_user_view.person.id;
  let local_instance_id = local_user_view.person.instance_id;
  let post_view = PostView::read(
    &mut context.pool(),
    post_id,
    Some(&local_user_view.local_user),
    local_instance_id,
    false,
  )
  .await?;

  let read_form = PostReadForm::new(post_id, person_id);
  PostActions::mark_as_read(&mut context.pool(), &read_form).await?;

  Ok(Json(PostResponse { post_view }))
}
