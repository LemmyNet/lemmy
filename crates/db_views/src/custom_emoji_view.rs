use crate::structs::CustomEmojiView;
use diesel::{result::Error, BelongingToDsl, ExpressionMethods, GroupedBy, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::LocalSiteId,
  schema::custom_emoji::{
    dsl::{custom_emoji, local_site_id},
    id,
  },
  source::{custom_emoji::CustomEmoji, custom_emoji_keyword::CustomEmojiKeyword},
  utils::{get_conn, DbPool},
};

impl CustomEmojiView {
  pub async fn get(pool: &DbPool, emoji_id: i32) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let emoji = custom_emoji
      .find(emoji_id)
      .first::<CustomEmoji>(conn)
      .await?;
    let keywords = CustomEmojiKeyword::belonging_to(&emoji)
      .load::<CustomEmojiKeyword>(conn)
      .await?;

    let view = CustomEmojiView {
      custom_emoji: emoji,
      keywords,
    };

    Ok(view)
  }

  pub async fn get_all(pool: &DbPool, for_local_site_id: LocalSiteId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let emojis = custom_emoji
      .filter(local_site_id.eq(for_local_site_id))
      .order(id.asc())
      .load::<CustomEmoji>(conn)
      .await?;
    let keywords = CustomEmojiKeyword::belonging_to(&emojis)
      .load::<CustomEmojiKeyword>(conn)
      .await?
      .grouped_by(&emojis);

    let views = emojis
      .into_iter()
      .zip(keywords)
      .map(|x| CustomEmojiView {
        custom_emoji: x.0,
        keywords: x.1,
      })
      .collect::<Vec<_>>();

    Ok(views)
  }
}
