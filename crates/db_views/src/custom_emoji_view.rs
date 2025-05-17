use crate::structs::CustomEmojiView;
use diesel::{result::Error, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::CustomEmojiId,
  schema::{custom_emoji, custom_emoji_keyword},
  source::{custom_emoji::CustomEmoji, custom_emoji_keyword::CustomEmojiKeyword},
  utils::{get_conn, limit_and_offset, DbPool},
};
use std::collections::HashMap;

type CustomEmojiTuple = (CustomEmoji, Option<CustomEmojiKeyword>);

impl CustomEmojiView {
  pub async fn get(pool: &mut DbPool<'_>, emoji_id: CustomEmojiId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let emojis = custom_emoji::table
      .find(emoji_id)
      .left_join(
        custom_emoji_keyword::table.on(custom_emoji_keyword::custom_emoji_id.eq(custom_emoji::id)),
      )
      .select((
        custom_emoji::all_columns,
        custom_emoji_keyword::all_columns.nullable(), // (or all the columns if you want)
      ))
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

    let mut query = custom_emoji::table
      .left_join(
        custom_emoji_keyword::table.on(custom_emoji_keyword::custom_emoji_id.eq(custom_emoji::id)),
      )
      .order(custom_emoji::category)
      .into_boxed();

    if !ignore_page_limits {
      let (limit, offset) = limit_and_offset(page, limit)?;
      query = query.limit(limit).offset(offset);
    }

    if let Some(category) = category {
      query = query.filter(custom_emoji::category.eq(category))
    }

    query = query.then_order_by(custom_emoji::id);

    let emojis = query
      .select((
        custom_emoji::all_columns,
        custom_emoji_keyword::all_columns.nullable(), // (or all the columns if you want)
      ))
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
