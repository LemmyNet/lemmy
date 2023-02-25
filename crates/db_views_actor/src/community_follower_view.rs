use crate::structs::CommunityFollowerView;
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::{community, community_follower, person},
  source::{community::Community, person::Person},
  traits::JoinView,
  utils::{get_conn, DbPool},
};

type CommunityFollowerViewTuple = (Community, Person);

impl CommunityFollowerView {
  pub async fn for_community(pool: &DbPool, community_id: CommunityId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = community_follower::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((community::all_columns, person::all_columns))
      .filter(community_follower::community_id.eq(community_id))
      .order_by(community::title)
      .load::<CommunityFollowerViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(Self::from_tuple).collect())
  }

  pub async fn for_person(pool: &DbPool, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = community_follower::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((community::all_columns, person::all_columns))
      .filter(community_follower::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .order_by(community::title)
      .load::<CommunityFollowerViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(Self::from_tuple).collect())
  }
}

impl JoinView for CommunityFollowerView {
  type JoinTuple = CommunityFollowerViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      community: a.0,
      follower: a.1,
    }
  }
}
