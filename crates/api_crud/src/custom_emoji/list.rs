use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  custom_emoji::{ListCustomEmojis, ListCustomEmojisResponse},
};
use lemmy_db_views::structs::{CustomEmojiView, LocalUserView};
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn list_custom_emojis(
  data: Query<ListCustomEmojis>,
  local_user_view: Option<LocalUserView>,
  context: Data<LemmyContext>,
) -> Result<Json<ListCustomEmojisResponse>, LemmyError> {
  let custom_emojis = CustomEmojiView::list(
    &mut context.pool(),
    &data.category,
    data.page,
    data.limit,
    data.ignore_page_limits.unwrap_or(false),
  )
  .await?;

  Ok(Json(ListCustomEmojisResponse { custom_emojis }))
}
