use activitypub_federation::config::Data;
use actix_web::web::{Json, Path};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::{newtypes::CustomEmojiId, source::custom_emoji::CustomEmoji, traits::Crud};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_utils::error::LemmyResult;

pub async fn delete_custom_emoji(
  id: Path<CustomEmojiId>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  CustomEmoji::delete(&mut context.pool(), id.into_inner()).await?;

  Ok(Json(SuccessResponse::default()))
}
