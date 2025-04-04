use crate::structs::CustomEmojiView;
use diesel::{
  dsl::Nullable,
  result::Error,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::CustomEmojiId,
  source::{custom_emoji::CustomEmoji, custom_emoji_keyword::CustomEmojiKeyword},
  utils::{get_conn, limit_and_offset, DbPool},
};
use lemmy_db_schema_file::schema::{custom_emoji, custom_emoji_keyword};
use std::collections::HashMap;

type SelectionType = (
  <custom_emoji::table as diesel::Table>::AllColumns,
  Nullable<<custom_emoji_keyword::table as diesel::Table>::AllColumns>,
);

fn selection() -> SelectionType {
  (
    custom_emoji::all_columns,
    custom_emoji_keyword::all_columns.nullable(), // (or all the columns if you want)
  )
}
type CustomEmojiTuple = (CustomEmoji, Option<CustomEmojiKeyword>);

// TODO this type is a mess, it should not be using vectors in a view.
impl CustomEmojiView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins() -> _ {
    custom_emoji::table.left_join(
      custom_emoji_keyword::table.on(custom_emoji_keyword::custom_emoji_id.eq(custom_emoji::id)),
    )
  }

  pub async fn get(pool: &mut DbPool<'_>, emoji_id: CustomEmojiId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let emojis = Self::joins()
      .filter(custom_emoji::id.eq(emoji_id))
      .select(selection())
      .load::<CustomEmojiTuple>(conn)
      .await?;
    if let Some(emoji) = CustomEmojiView::from_tuple_to_vec(emojis)
      .into_iter()
      .next()
    {
      Ok(emoji)
    } else {
      Err(diesel::result::Error::NotFound)
    }
  }

  pub async fn list(
    pool: &mut DbPool<'_>,
    category: &Option<String>,
    page: Option<i64>,
    limit: Option<i64>,
    ignore_page_limits: bool,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;

    let mut query = Self::joins().into_boxed();

    if !ignore_page_limits {
      let (limit, offset) = limit_and_offset(page, limit)?;
      query = query.limit(limit).offset(offset);
    }

    if let Some(category) = category {
      query = query.filter(custom_emoji::category.eq(category))
    }

    let emojis = query
      .select(selection())
      .order(custom_emoji::category)
      .then_order_by(custom_emoji::id)
      .load::<CustomEmojiTuple>(conn)
      .await?;

    Ok(CustomEmojiView::from_tuple_to_vec(emojis))
  }

  fn from_tuple_to_vec(items: Vec<CustomEmojiTuple>) -> Vec<Self> {
    let mut result = Vec::new();
    let mut hash: HashMap<CustomEmojiId, Vec<CustomEmojiKeyword>> = HashMap::new();
    for (emoji, keyword) in &items {
      let emoji_id: CustomEmojiId = emoji.id;
      if let std::collections::hash_map::Entry::Vacant(e) = hash.entry(emoji_id) {
        e.insert(Vec::new());
        result.push(CustomEmojiView {
          custom_emoji: emoji.clone(),
          keywords: Vec::new(),
        })
      }
      if let Some(item_keyword) = &keyword {
        if let Some(keywords) = hash.get_mut(&emoji_id) {
          keywords.push(item_keyword.clone())
        }
      }
    }
    for emoji in &mut result {
      if let Some(keywords) = hash.get_mut(&emoji.custom_emoji.id) {
        emoji.keywords.clone_from(keywords);
      }
    }
    result
  }
}
