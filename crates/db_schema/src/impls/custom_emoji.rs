use crate::{
  newtypes::CustomEmojiId,
  source::{
    custom_emoji::{CustomEmoji, CustomEmojiInsertForm, CustomEmojiUpdateForm},
    custom_emoji_keyword::{CustomEmojiKeyword, CustomEmojiKeywordInsertForm},
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{
  custom_emoji::dsl::custom_emoji,
  custom_emoji_keyword::dsl::{custom_emoji_id, custom_emoji_keyword},
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Crud for CustomEmoji {
  type InsertForm = CustomEmojiInsertForm;
  type UpdateForm = CustomEmojiUpdateForm;
  type IdType = CustomEmojiId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(custom_emoji)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreateEmoji)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    emoji_id: Self::IdType,
    new_custom_emoji: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(custom_emoji.find(emoji_id))
      .set(new_custom_emoji)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdateEmoji)
  }
}

impl CustomEmojiKeyword {
  pub async fn create(
    pool: &mut DbPool<'_>,
    form: Vec<CustomEmojiKeywordInsertForm>,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    insert_into(custom_emoji_keyword)
      .values(form)
      .get_results::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreateEmoji)
  }
  pub async fn delete(pool: &mut DbPool<'_>, emoji_id: CustomEmojiId) -> LemmyResult<usize> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(custom_emoji_keyword.filter(custom_emoji_id.eq(emoji_id)))
      .execute(conn)
      .await
      .with_lemmy_type(LemmyErrorType::Deleted)
  }
}
