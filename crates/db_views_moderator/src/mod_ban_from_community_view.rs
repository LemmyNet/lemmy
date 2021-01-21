use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{community, mod_ban_from_community, user_, user_alias_1},
  source::{
    community::{Community, CommunitySafe},
    moderator::ModBanFromCommunity,
    user::{UserAlias1, UserSafe, UserSafeAlias1, User_},
  },
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ModBanFromCommunityView {
  pub mod_ban_from_community: ModBanFromCommunity,
  pub moderator: UserSafe,
  pub community: CommunitySafe,
  pub banned_user: UserSafeAlias1,
}

type ModBanFromCommunityViewTuple = (ModBanFromCommunity, UserSafe, CommunitySafe, UserSafeAlias1);

impl ModBanFromCommunityView {
  pub fn list(
    conn: &PgConnection,
    community_id: Option<i32>,
    mod_user_id: Option<i32>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_ban_from_community::table
      .inner_join(user_::table.on(mod_ban_from_community::mod_user_id.eq(user_::id)))
      .inner_join(community::table)
      .inner_join(user_alias_1::table.on(mod_ban_from_community::other_user_id.eq(user_::id)))
      .select((
        mod_ban_from_community::all_columns,
        User_::safe_columns_tuple(),
        Community::safe_columns_tuple(),
        UserAlias1::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(mod_user_id) = mod_user_id {
      query = query.filter(mod_ban_from_community::mod_user_id.eq(mod_user_id));
    };

    if let Some(community_id) = community_id {
      query = query.filter(mod_ban_from_community::community_id.eq(community_id));
    };

    let (limit, offset) = limit_and_offset(page, limit);

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_ban_from_community::when_.desc())
      .load::<ModBanFromCommunityViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for ModBanFromCommunityView {
  type DbTuple = ModBanFromCommunityViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        mod_ban_from_community: a.0.to_owned(),
        moderator: a.1.to_owned(),
        community: a.2.to_owned(),
        banned_user: a.3.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
