use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  custom_emoji::{CreateCustomEmoji, CustomEmojiResponse},
  utils::is_admin,
};
use lemmy_db_schema::source::{
  custom_emoji::{CustomEmoji, CustomEmojiInsertForm},
  custom_emoji_keyword::{CustomEmojiKeyword, CustomEmojiKeywordInsertForm},
  local_site::LocalSite,
};
use lemmy_db_views::structs::{CustomEmojiView, LocalUserView};
use lemmy_utils::error::LemmyError;

#[tracing::instrument(skip(context))]
pub async fn create_custom_emoji(
  data: Json<CreateCustomEmoji>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<CustomEmojiResponse>, LemmyError> {
  let local_site = LocalSite::read(&mut context.pool()).await?;
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let emoji_form = CustomEmojiInsertForm::builder()
    .local_site_id(local_site.id)
    .shortcode(data.shortcode.to_lowercase().trim().to_string())
    .alt_text(data.alt_text.to_string())
    .category(data.category.to_string())
    .image_url(data.clone().image_url.into())
    .build();
  let emoji = CustomEmoji::create(&mut context.pool(), &emoji_form).await?;
  let mut keywords = vec![];
  for keyword in &data.keywords {
    let keyword_form = CustomEmojiKeywordInsertForm::builder()
      .custom_emoji_id(emoji.id)
      .keyword(keyword.to_lowercase().trim().to_string())
      .build();
    keywords.push(keyword_form);
  }
  CustomEmojiKeyword::create(&mut context.pool(), keywords).await?;
  let view = CustomEmojiView::get(&mut context.pool(), emoji.id).await?;
  Ok(Json(CustomEmojiResponse { custom_emoji: view }))
}
