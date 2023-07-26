use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  custom_emoji::{CustomEmojiResponse, EditCustomEmoji},
  utils::{is_admin, local_user_view_from_jwt, sanitize_html},
};
use lemmy_db_schema::source::{
  custom_emoji::{CustomEmoji, CustomEmojiUpdateForm},
  custom_emoji_keyword::{CustomEmojiKeyword, CustomEmojiKeywordInsertForm},
  local_site::LocalSite,
};
use lemmy_db_views::structs::CustomEmojiView;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditCustomEmoji {
  type Response = CustomEmojiResponse;

  #[tracing::instrument(skip(self, context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CustomEmojiResponse, LemmyError> {
    let data: &EditCustomEmoji = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let local_site = LocalSite::read(context.pool()).await?;
    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let alt_text = sanitize_html(&data.alt_text);
    let category = sanitize_html(&data.category);

    let emoji_form = CustomEmojiUpdateForm::builder()
      .local_site_id(local_site.id)
      .alt_text(alt_text)
      .category(category)
      .image_url(data.clone().image_url.into())
      .build();
    let emoji = CustomEmoji::update(context.pool(), data.id, &emoji_form).await?;
    CustomEmojiKeyword::delete(context.pool(), data.id).await?;
    let mut keywords = vec![];
    for keyword in &data.keywords {
      let keyword_form = CustomEmojiKeywordInsertForm::builder()
        .custom_emoji_id(emoji.id)
        .keyword(keyword.to_lowercase().trim().to_string())
        .build();
      keywords.push(keyword_form);
    }
    CustomEmojiKeyword::create(context.pool(), keywords).await?;
    let view = CustomEmojiView::get(context.pool(), emoji.id).await?;
    Ok(CustomEmojiResponse { custom_emoji: view })
  }
}
