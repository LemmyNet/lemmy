use crate::{
  schema::community_hide_from_feeds::dsl::{community_hide_from_feeds, community_id, person_id},
  source::community_hide_from_feeds::{CommunityHideFromFeeds, CommunityHideFromFeedsForm},
  traits::HideableFromFeeds,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl HideableFromFeeds for CommunityHideFromFeeds {
  type Form = CommunityHideFromFeedsForm;
  async fn hide_from_feeds(
    pool: &DbPool,
    community_hide_from_feeds_form: &Self::Form,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(community_hide_from_feeds)
      .values(community_hide_from_feeds_form)
      .on_conflict((person_id, community_id))
      .do_update()
      .set(community_hide_from_feeds_form)
      .get_result::<Self>(conn)
      .await
  }
  async fn unhide_from_feeds(
    pool: &DbPool,
    community_hide_from_feeds_form: &Self::Form,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      community_hide_from_feeds
        .filter(person_id.eq(community_hide_from_feeds_form.person_id))
        .filter(community_id.eq(community_hide_from_feeds_form.community_id)),
    )
    .execute(conn)
    .await
  }
}
