use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  custom_emoji::{DeleteCustomEmoji, DeleteCustomEmojiResponse},
  utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::source::custom_emoji::CustomEmoji;
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn delete_custom_emoji(
  data: Json<DeleteCustomEmoji>,
  context: Data<LemmyContext>,
) -> Result<Json<DeleteCustomEmojiResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

  // Make sure user is an admin
  is_admin(&local_user_view)?;
  CustomEmoji::delete(&mut context.pool(), data.id).await?;
  Ok(Json(DeleteCustomEmojiResponse {
    id: data.id,
    success: true,
  }))
}
