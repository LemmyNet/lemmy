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
  utils::{DbPool, DbPoolRef, RunQueryDsl},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};

impl CustomEmoji {
  pub async fn create(pool: DbPoolRef<'_>, form: &CustomEmojiInsertForm) -> Result<Self, Error> {
    let conn = pool;
    insert_into(custom_emoji)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn update(
    pool: DbPoolRef<'_>,
    emoji_id: CustomEmojiId,
    form: &CustomEmojiUpdateForm,
  ) -> Result<Self, Error> {
    let conn = pool;
    diesel::update(custom_emoji.find(emoji_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  pub async fn delete(pool: DbPoolRef<'_>, emoji_id: CustomEmojiId) -> Result<usize, Error> {
    let conn = pool;
    diesel::delete(custom_emoji.find(emoji_id))
      .execute(conn)
      .await
  }
}

impl CustomEmojiKeyword {
  pub async fn create(
    pool: DbPoolRef<'_>,
    form: Vec<CustomEmojiKeywordInsertForm>,
  ) -> Result<Vec<Self>, Error> {
    let conn = pool;
    insert_into(custom_emoji_keyword)
      .values(form)
      .get_results::<Self>(conn)
      .await
  }
  pub async fn delete(pool: DbPoolRef<'_>, emoji_id: CustomEmojiId) -> Result<usize, Error> {
    let conn = pool;
    diesel::delete(custom_emoji_keyword.filter(custom_emoji_id.eq(emoji_id)))
      .execute(conn)
      .await
  }
}
