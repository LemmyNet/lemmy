use crate::{
  community::{Community, CommunitySafe},
  schema::{community, community_moderator, user_},
  user::{UserSafe, User_},
  ToSafe,
};
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CommunityModeratorView {
  pub community: CommunitySafe,
  pub moderator: UserSafe,
}

impl CommunityModeratorView {
  pub fn for_community(conn: &PgConnection, for_community_id: i32) -> Result<Vec<Self>, Error> {
    let res = community_moderator::table
      .inner_join(community::table)
      .inner_join(user_::table)
      .select((Community::safe_columns_tuple(), User_::safe_columns_tuple()))
      .filter(community_moderator::community_id.eq(for_community_id))
      .order_by(community_moderator::published)
      .load::<(CommunitySafe, UserSafe)>(conn)?;

    Ok(to_vec(res))
  }

  pub fn for_user(conn: &PgConnection, for_user_id: i32) -> Result<Vec<Self>, Error> {
    let res = community_moderator::table
      .inner_join(community::table)
      .inner_join(user_::table)
      .select((Community::safe_columns_tuple(), User_::safe_columns_tuple()))
      .filter(community_moderator::user_id.eq(for_user_id))
      .order_by(community_moderator::published)
      .load::<(CommunitySafe, UserSafe)>(conn)?;

    Ok(to_vec(res))
  }
}

fn to_vec(users: Vec<(CommunitySafe, UserSafe)>) -> Vec<CommunityModeratorView> {
  users
    .iter()
    .map(|a| CommunityModeratorView {
      community: a.0.to_owned(),
      moderator: a.1.to_owned(),
    })
    .collect::<Vec<CommunityModeratorView>>()
}
