use crate::{
  newtypes::CustomEmojiId,
  schema::{
    custom_emoji::dsl::custom_emoji,
    custom_emoji_keyword::dsl::{custom_emoji_id, custom_emoji_keyword},
  },
  source::{
    custom_emoji::{CustomEmoji, CustomEmojiInsertForm, CustomEmojiUpdateForm},
    custom_emoji_keyword::{CustomEmojiKeyword, CustomEmojiKeywordInsertForm},
  },
  utils::GetConn,
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use lemmy_db_schema::utils::RunQueryDsl;

impl CustomEmoji {
  pub async fn create(mut conn: impl GetConn, form: &CustomEmojiInsertForm) -> Result<Self, Error> {
    insert_into(custom_emoji)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn update(
    mut conn: impl GetConn,
    emoji_id: CustomEmojiId,
    form: &CustomEmojiUpdateForm,
  ) -> Result<Self, Error> {
    diesel::update(custom_emoji.find(emoji_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn delete(mut conn: impl GetConn, emoji_id: CustomEmojiId) -> Result<usize, Error> {
    diesel::delete(custom_emoji.find(emoji_id))
      .execute(conn)
      .await
  }
}

impl CustomEmojiKeyword {
  pub async fn create(
    mut conn: impl GetConn,
    form: Vec<CustomEmojiKeywordInsertForm>,
  ) -> Result<Vec<Self>, Error> {
    insert_into(custom_emoji_keyword)
      .values(form)
      .get_results::<Self>(conn)
      .await
  }
  pub async fn delete(mut conn: impl GetConn, emoji_id: CustomEmojiId) -> Result<usize, Error> {
    diesel::delete(custom_emoji_keyword.filter(custom_emoji_id.eq(emoji_id)))
      .execute(conn)
      .await
  }
}
