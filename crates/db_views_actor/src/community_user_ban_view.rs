use diesel::{result::Error, *};
use lemmy_db_queries::ToSafe;
use lemmy_db_schema::{
  schema::{community, community_user_ban, user_},
  source::{
    community::{Community, CommunitySafe},
    user::{UserSafe, User_},
  },
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CommunityUserBanView {
  pub community: CommunitySafe,
  pub user: UserSafe,
}

impl CommunityUserBanView {
  pub fn get(
    conn: &PgConnection,
    from_user_id: i32,
    from_community_id: i32,
  ) -> Result<Self, Error> {
    let (community, user) = community_user_ban::table
      .inner_join(community::table)
      .inner_join(user_::table)
      .select((Community::safe_columns_tuple(), User_::safe_columns_tuple()))
      .filter(community_user_ban::community_id.eq(from_community_id))
      .filter(community_user_ban::user_id.eq(from_user_id))
      .order_by(community_user_ban::published)
      .first::<(CommunitySafe, UserSafe)>(conn)?;

    Ok(CommunityUserBanView { community, user })
  }
}
