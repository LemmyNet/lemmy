use crate::structs::CommunityModeratorView;
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::{community, community_moderator, person},
  source::{community::Community, person::Person},
  traits::JoinView,
  utils::{DbPool, GetConn},
};

type CommunityModeratorViewTuple = (Community, Person);

impl CommunityModeratorView {
  pub async fn for_community(
    mut pool: &mut impl GetConn,
    community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut *pool.get_conn().await?;
    let res = community_moderator::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((community::all_columns, person::all_columns))
      .filter(community_moderator::community_id.eq(community_id))
      .load::<CommunityModeratorViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(Self::from_tuple).collect())
  }

  pub async fn for_person(
    mut pool: &mut impl GetConn,
    person_id: PersonId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut *pool.get_conn().await?;
    let res = community_moderator::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((community::all_columns, person::all_columns))
      .filter(community_moderator::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .load::<CommunityModeratorViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(Self::from_tuple).collect())
  }

  /// Finds all communities first mods / creators
  /// Ideally this should be a group by, but diesel doesn't support it yet
  pub async fn get_community_first_mods(mut pool: &mut impl GetConn) -> Result<Vec<Self>, Error> {
    let conn = &mut *pool.get_conn().await?;
    let res = community_moderator::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((community::all_columns, person::all_columns))
      // A hacky workaround instead of group_bys
      // https://stackoverflow.com/questions/24042359/how-to-join-only-one-row-in-joined-table-with-postgres
      .distinct_on(community_moderator::community_id)
      .order_by((
        community_moderator::community_id,
        community_moderator::person_id,
      ))
      .load::<CommunityModeratorViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(Self::from_tuple).collect())
  }
}

impl JoinView for CommunityModeratorView {
  type JoinTuple = CommunityModeratorViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      community: a.0,
      moderator: a.1,
    }
  }
}
