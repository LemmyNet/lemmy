use actix_web::web::{Data, Json, Query};
use lemmy_api_common::{
  context::LemmyContext,
  custom_emoji::{ListCustomEmojis, ListCustomEmojisResponse},
};
use lemmy_db_views::structs::{CustomEmojiView, LocalUserView, SiteView};
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn list_custom_emojis(
  data: Query<ListCustomEmojis>,
  local_user_view: Option<LocalUserView>,
  context: Data<LemmyContext>,
) -> Result<Json<ListCustomEmojisResponse>, LemmyError> {
  let local_site = SiteView::read_local(&mut context.pool()).await?;
  let custom_emojis = CustomEmojiView::list(
    &mut context.pool(),
    local_site.local_site.id,
    &data.category,
    data.page,
    data.limit,
  )
  .await?;

  Ok(Json(ListCustomEmojisResponse { custom_emojis }))
}
