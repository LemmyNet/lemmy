use diesel::{result::Error, *};
use lemmy_db_queries::{ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{community, community_follower, user_},
  source::{
    community::{Community, CommunitySafe},
    user::{UserSafe, User_},
  },
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CommunityFollowerView {
  pub community: CommunitySafe,
  pub follower: UserSafe,
}

type CommunityFollowerViewTuple = (CommunitySafe, UserSafe);

impl CommunityFollowerView {
  pub fn for_community(conn: &PgConnection, community_id: i32) -> Result<Vec<Self>, Error> {
    let res = community_follower::table
      .inner_join(community::table)
      .inner_join(user_::table)
      .select((Community::safe_columns_tuple(), User_::safe_columns_tuple()))
      .filter(community_follower::community_id.eq(community_id))
      .order_by(community_follower::published)
      .load::<CommunityFollowerViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }

  pub fn for_user(conn: &PgConnection, user_id: i32) -> Result<Vec<Self>, Error> {
    let res = community_follower::table
      .inner_join(community::table)
      .inner_join(user_::table)
      .select((Community::safe_columns_tuple(), User_::safe_columns_tuple()))
      .filter(community_follower::user_id.eq(user_id))
      .order_by(community_follower::published)
      .load::<CommunityFollowerViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for CommunityFollowerView {
  type DbTuple = CommunityFollowerViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        community: a.0.to_owned(),
        follower: a.1.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
