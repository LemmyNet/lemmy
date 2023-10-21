use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  custom_emoji::DeleteCustomEmoji,
  utils::is_admin,
  SuccessResponse,
};
use lemmy_db_schema::source::custom_emoji::CustomEmoji;
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn delete_custom_emoji(
  data: Json<DeleteCustomEmoji>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<SuccessResponse>, LemmyError> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  CustomEmoji::delete(&mut context.pool(), data.id).await?;

  Ok(Json(SuccessResponse::default()))
}
