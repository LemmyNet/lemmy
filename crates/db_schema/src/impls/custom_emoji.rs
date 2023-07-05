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
  utils::{DbPool, GetConn},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl CustomEmoji {
  pub async fn create(
    mut pool: &mut impl GetConn,
    form: &CustomEmojiInsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut *pool.get_conn().await?;
    insert_into(custom_emoji)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn update(
    mut pool: &mut impl GetConn,
    emoji_id: CustomEmojiId,
    form: &CustomEmojiUpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut *pool.get_conn().await?;
    diesel::update(custom_emoji.find(emoji_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn delete(
    mut pool: &mut impl GetConn,
    emoji_id: CustomEmojiId,
  ) -> Result<usize, Error> {
    let conn = &mut *pool.get_conn().await?;
    diesel::delete(custom_emoji.find(emoji_id))
      .execute(conn)
      .await
  }
}

impl CustomEmojiKeyword {
  pub async fn create(
    mut pool: &mut impl GetConn,
    form: Vec<CustomEmojiKeywordInsertForm>,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut *pool.get_conn().await?;
    insert_into(custom_emoji_keyword)
      .values(form)
      .get_results::<Self>(conn)
      .await
  }
  pub async fn delete(
    mut pool: &mut impl GetConn,
    emoji_id: CustomEmojiId,
  ) -> Result<usize, Error> {
    let conn = &mut *pool.get_conn().await?;
    diesel::delete(custom_emoji_keyword.filter(custom_emoji_id.eq(emoji_id)))
      .execute(conn)
      .await
  }
}
