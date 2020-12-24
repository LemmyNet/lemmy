use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{community, mod_remove_community, user_},
  source::{
    community::{Community, CommunitySafe},
    moderator::ModRemoveCommunity,
    user::{UserSafe, User_},
  },
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ModRemoveCommunityView {
  pub mod_remove_community: ModRemoveCommunity,
  pub moderator: UserSafe,
  pub community: CommunitySafe,
}

type ModRemoveCommunityTuple = (ModRemoveCommunity, UserSafe, CommunitySafe);

impl ModRemoveCommunityView {
  pub fn list(
    conn: &PgConnection,
    mod_user_id: Option<i32>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_remove_community::table
      .inner_join(user_::table)
      .inner_join(community::table)
      .select((
        mod_remove_community::all_columns,
        User_::safe_columns_tuple(),
        Community::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(mod_user_id) = mod_user_id {
      query = query.filter(mod_remove_community::mod_user_id.eq(mod_user_id));
    };

    let (limit, offset) = limit_and_offset(page, limit);

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_remove_community::when_.desc())
      .load::<ModRemoveCommunityTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for ModRemoveCommunityView {
  type DbTuple = ModRemoveCommunityTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        mod_remove_community: a.0.to_owned(),
        moderator: a.1.to_owned(),
        community: a.2.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
