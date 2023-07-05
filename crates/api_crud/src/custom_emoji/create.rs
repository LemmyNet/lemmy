use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  custom_emoji::{CreateCustomEmoji, CustomEmojiResponse},
  utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::source::{
  custom_emoji::{CustomEmoji, CustomEmojiInsertForm},
  custom_emoji_keyword::{CustomEmojiKeyword, CustomEmojiKeywordInsertForm},
  local_site::LocalSite,
};
use lemmy_db_views::structs::CustomEmojiView;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreateCustomEmoji {
  type Response = CustomEmojiResponse;

  #[tracing::instrument(skip(self, context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CustomEmojiResponse, LemmyError> {
    let data: &CreateCustomEmoji = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let local_site = LocalSite::read(context.pool()).await?;
    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let emoji_form = CustomEmojiInsertForm::builder()
      .local_site_id(local_site.id)
      .shortcode(data.shortcode.to_lowercase().trim().to_string())
      .alt_text(data.alt_text.to_string())
      .category(data.category.to_string())
      .image_url(data.clone().image_url.into())
      .build();
    let emoji = CustomEmoji::create(context.pool(), &emoji_form).await?;
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
