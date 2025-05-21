use actix_web::web::{Data, Json, Query};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_views_custom_emoji::CustomEmojiView;
use lemmy_db_views_list_custom_emojis::ListCustomEmojis;
use lemmy_db_views_list_custom_emojis_response::ListCustomEmojisResponse;
use lemmy_utils::error::LemmyError;

pub async fn list_custom_emojis(
  data: Query<ListCustomEmojis>,
  context: Data<LemmyContext>,
) -> Result<Json<ListCustomEmojisResponse>, LemmyError> {
  let custom_emojis = CustomEmojiView::list(&mut context.pool(), &data.category).await?;

  Ok(Json(ListCustomEmojisResponse { custom_emojis }))
}
