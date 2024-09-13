use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  custom_emoji::{CustomEmojiResponse, EditCustomEmoji},
  utils::is_admin,
};
use lemmy_db_schema::source::{
  custom_emoji::{CustomEmoji, CustomEmojiUpdateForm},
  custom_emoji_keyword::{CustomEmojiKeyword, CustomEmojiKeywordInsertForm},
  local_site::LocalSite,
};
use lemmy_db_views::structs::{CustomEmojiView, LocalUserView};
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn update_custom_emoji(
  data: Json<EditCustomEmoji>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CustomEmojiResponse>> {
  let local_site = LocalSite::read(&mut context.pool()).await?;
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let emoji_form = CustomEmojiUpdateForm::new(
    local_site.id,
    data.clone().image_url.into(),
    data.alt_text.to_string(),
    data.category.to_string(),
  );
  let emoji = CustomEmoji::update(&mut context.pool(), data.id, &emoji_form).await?;
  CustomEmojiKeyword::delete(&mut context.pool(), data.id).await?;
  let mut keywords = vec![];
  for keyword in &data.keywords {
    let keyword_form =
      CustomEmojiKeywordInsertForm::new(emoji.id, keyword.to_lowercase().trim().to_string());
    keywords.push(keyword_form);
  }
  CustomEmojiKeyword::create(&mut context.pool(), keywords).await?;
  let view = CustomEmojiView::get(&mut context.pool(), emoji.id).await?;
  Ok(Json(CustomEmojiResponse { custom_emoji: view }))
}
