use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{community, mod_add_community, user_, user_alias_1},
  source::{
    community::{Community, CommunitySafe},
    moderator::ModAddCommunity,
    user::{UserAlias1, UserSafe, UserSafeAlias1, User_},
  },
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ModAddCommunityView {
  pub mod_add_community: ModAddCommunity,
  pub moderator: UserSafe,
  pub community: CommunitySafe,
  pub modded_user: UserSafeAlias1,
}

type ModAddCommunityViewTuple = (ModAddCommunity, UserSafe, CommunitySafe, UserSafeAlias1);

impl ModAddCommunityView {
  pub fn list(
    conn: &PgConnection,
    community_id: Option<i32>,
    mod_user_id: Option<i32>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_add_community::table
      .inner_join(user_::table.on(mod_add_community::mod_user_id.eq(user_::id)))
      .inner_join(community::table)
      .inner_join(user_alias_1::table.on(mod_add_community::other_user_id.eq(user_::id)))
      .select((
        mod_add_community::all_columns,
        User_::safe_columns_tuple(),
        Community::safe_columns_tuple(),
        UserAlias1::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(mod_user_id) = mod_user_id {
      query = query.filter(mod_add_community::mod_user_id.eq(mod_user_id));
    };

    if let Some(community_id) = community_id {
      query = query.filter(mod_add_community::community_id.eq(community_id));
    };

    let (limit, offset) = limit_and_offset(page, limit);

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_add_community::when_.desc())
      .load::<ModAddCommunityViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for ModAddCommunityView {
  type DbTuple = ModAddCommunityViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        mod_add_community: a.0.to_owned(),
        moderator: a.1.to_owned(),
        community: a.2.to_owned(),
        modded_user: a.3.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
