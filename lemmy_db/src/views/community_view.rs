use crate::{
  aggregates::community_aggregates::CommunityAggregates,
  category::Category,
  community::{Community, CommunityFollower},
  schema::{category, community, community_aggregates, community_follower, user_},
  user::{UserSafe, User_},
};
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CommunityView {
  pub community: Community,
  pub creator: UserSafe,
  pub category: Category,
  pub subscribed: bool,
  pub counts: CommunityAggregates,
}

impl CommunityView {
  pub fn read(
    conn: &PgConnection,
    community_id: i32,
    my_user_id: Option<i32>,
  ) -> Result<Self, Error> {
    let subscribed = match my_user_id {
      Some(user_id) => {
        let res = community_follower::table
          .filter(community_follower::community_id.eq(community_id))
          .filter(community_follower::user_id.eq(user_id))
          .get_result::<CommunityFollower>(conn);
        res.is_ok()
      }
      None => false,
    };

    let (community, creator, category, counts) = community::table
      .find(community_id)
      .inner_join(user_::table)
      .inner_join(category::table)
      .inner_join(community_aggregates::table)
      .first::<(Community, User_, Category, CommunityAggregates)>(conn)?;

    Ok(CommunityView {
      community,
      creator: creator.to_safe(),
      category,
      subscribed,
      counts,
    })
  }
}
