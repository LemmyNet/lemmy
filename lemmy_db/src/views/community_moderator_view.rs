use crate::{
  community::{Community, CommunitySafe},
  schema::{community, community_moderator, user_},
  user::{UserSafe, User_},
  views::ViewToVec,
  ToSafe,
};
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CommunityModeratorView {
  pub community: CommunitySafe,
  pub moderator: UserSafe,
}

type CommunityModeratorViewTuple = (CommunitySafe, UserSafe);

impl CommunityModeratorView {
  pub fn for_community(conn: &PgConnection, for_community_id: i32) -> Result<Vec<Self>, Error> {
    let res = community_moderator::table
      .inner_join(community::table)
      .inner_join(user_::table)
      .select((Community::safe_columns_tuple(), User_::safe_columns_tuple()))
      .filter(community_moderator::community_id.eq(for_community_id))
      .order_by(community_moderator::published)
      .load::<CommunityModeratorViewTuple>(conn)?;

    Ok(Self::to_vec(res))
  }

  pub fn for_user(conn: &PgConnection, for_user_id: i32) -> Result<Vec<Self>, Error> {
    let res = community_moderator::table
      .inner_join(community::table)
      .inner_join(user_::table)
      .select((Community::safe_columns_tuple(), User_::safe_columns_tuple()))
      .filter(community_moderator::user_id.eq(for_user_id))
      .order_by(community_moderator::published)
      .load::<CommunityModeratorViewTuple>(conn)?;

    Ok(Self::to_vec(res))
  }
}

impl ViewToVec for CommunityModeratorView {
  type DbTuple = CommunityModeratorViewTuple;
  fn to_vec(community_moderators: Vec<Self::DbTuple>) -> Vec<Self> {
    community_moderators
      .iter()
      .map(|a| Self {
        community: a.0.to_owned(),
        moderator: a.1.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
