use crate::structs::CommunityHideFromFeedsView;
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{community, community_hide_from_feeds, person},
  source::{community::Community, person::Person},
  traits::JoinView,
  utils::{get_conn, DbPool},
};

type CommunityHideFromFeedsViewTuple = (Person, Community);

impl CommunityHideFromFeedsView {
  pub async fn for_person(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = community_hide_from_feeds::table
      .inner_join(person::table)
      .inner_join(community::table)
      .select((person::all_columns, community::all_columns))
      .filter(community_hide_from_feeds::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .order_by(community_hide_from_feeds::published)
      .load::<CommunityHideFromFeedsViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(Self::from_tuple).collect())
  }
}

impl JoinView for CommunityHideFromFeedsView {
  type JoinTuple = CommunityHideFromFeedsViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      person: a.0,
      community: a.1,
    }
  }
}
