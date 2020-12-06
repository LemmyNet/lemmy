use crate::{
  community::{Community, CommunitySafe},
  schema::{community, community_follower, user_},
  user::{UserSafe, User_},
  ToSafe,
};
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CommunityFollowerView {
  pub community: CommunitySafe,
  pub follower: UserSafe,
}

impl CommunityFollowerView {
  pub fn for_community(conn: &PgConnection, for_community_id: i32) -> Result<Vec<Self>, Error> {
    let res = community_follower::table
      .inner_join(community::table)
      .inner_join(user_::table)
      .select((Community::safe_columns_tuple(), User_::safe_columns_tuple()))
      .filter(community_follower::community_id.eq(for_community_id))
      .order_by(community_follower::published)
      .load::<(CommunitySafe, UserSafe)>(conn)?;

    Ok(to_vec(res))
  }

  pub fn for_user(conn: &PgConnection, for_user_id: i32) -> Result<Vec<Self>, Error> {
    let res = community_follower::table
      .inner_join(community::table)
      .inner_join(user_::table)
      .select((Community::safe_columns_tuple(), User_::safe_columns_tuple()))
      .filter(community_follower::user_id.eq(for_user_id))
      .order_by(community_follower::published)
      .load::<(CommunitySafe, UserSafe)>(conn)?;

    Ok(to_vec(res))
  }
}

fn to_vec(users: Vec<(CommunitySafe, UserSafe)>) -> Vec<CommunityFollowerView> {
  users
    .iter()
    .map(|a| CommunityFollowerView {
      community: a.0.to_owned(),
      follower: a.1.to_owned(),
    })
    .collect::<Vec<CommunityFollowerView>>()
}
