use actix_web::web::{Data, Json};
use lemmy_api_common::{context::LemmyContext, post::SubscribePost, SuccessResponse};
use lemmy_db_schema::source::post::{PostActions, PostSubscribeForm};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn subscribe_post(
  data: Json<SubscribePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  let form = PostSubscribeForm::new(data.post_id, local_user_view.person.id);

  if data.subscribe {
    PostActions::subscribe(&mut context.pool(), &form).await?;
  } else {
    PostActions::unsubscribe(&mut context.pool(), &form).await?;
  }

  Ok(Json(SuccessResponse::default()))
}
