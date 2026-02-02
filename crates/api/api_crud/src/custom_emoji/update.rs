use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::{
  custom_emoji::{CustomEmoji, CustomEmojiUpdateForm},
  custom_emoji_keyword::CustomEmojiKeyword,
};
use lemmy_db_views_custom_emoji::{
  CustomEmojiView,
  api::{CustomEmojiResponse, EditCustomEmoji},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::LemmyResult;

pub async fn edit_custom_emoji(
  Json(data): Json<EditCustomEmoji>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CustomEmojiResponse>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let emoji_form = CustomEmojiUpdateForm {
    image_url: data.image_url.clone(),
    shortcode: data
      .shortcode
      .clone()
      .map(|s| s.to_lowercase().trim().to_string()),
    alt_text: data.alt_text.clone(),
    category: data.category.clone(),
  };
  let emoji = CustomEmoji::update(&mut context.pool(), data.id, &emoji_form).await?;

  // Delete the existing keywords, and recreate
  if let Some(keywords) = &data.keywords {
    CustomEmojiKeyword::delete(&mut context.pool(), data.id).await?;
    CustomEmojiKeyword::create_from_keywords(&mut context.pool(), emoji.id, keywords).await?;
  }

  let view = CustomEmojiView::get(&mut context.pool(), emoji.id).await?;
  Ok(Json(CustomEmojiResponse { custom_emoji: view }))
}
