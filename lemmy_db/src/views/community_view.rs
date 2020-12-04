use crate::{
  category::Category,
  community::{Community, CommunityFollower},
  schema::{category, community, community_follower, user_},
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
}

// creator_actor_id -> Text,
// creator_local -> Bool,
// creator_name -> Varchar,
// creator_preferred_username -> Nullable<Varchar>,
// creator_avatar -> Nullable<Text>,
// category_name -> Varchar,
// number_of_subscribers -> BigInt,
// number_of_posts -> BigInt,
// number_of_comments -> BigInt,
// hot_rank -> Int4,
// user_id -> Nullable<Int4>,
// subscribed -> Nullable<Bool>,

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

    let (community, creator, category) = community::table
      .find(community_id)
      .inner_join(user_::table)
      .inner_join(category::table)
      .first::<(Community, User_, Category)>(conn)?;

    Ok(CommunityView {
      community,
      creator: creator.to_safe(),
      category,
      subscribed,
    })
  }
}
