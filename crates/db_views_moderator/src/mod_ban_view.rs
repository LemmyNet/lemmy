use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{mod_ban, user_, user_alias_1},
  source::{
    moderator::ModBan,
    user::{UserAlias1, UserSafe, UserSafeAlias1, User_},
  },
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ModBanView {
  pub mod_ban: ModBan,
  pub moderator: UserSafe,
  pub banned_user: UserSafeAlias1,
}

type ModBanViewTuple = (ModBan, UserSafe, UserSafeAlias1);

impl ModBanView {
  pub fn list(
    conn: &PgConnection,
    mod_user_id: Option<i32>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_ban::table
      .inner_join(user_::table.on(mod_ban::mod_user_id.eq(user_::id)))
      .inner_join(user_alias_1::table.on(mod_ban::other_user_id.eq(user_alias_1::id)))
      .select((
        mod_ban::all_columns,
        User_::safe_columns_tuple(),
        UserAlias1::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(mod_user_id) = mod_user_id {
      query = query.filter(mod_ban::mod_user_id.eq(mod_user_id));
    };

    let (limit, offset) = limit_and_offset(page, limit);

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_ban::when_.desc())
      .load::<ModBanViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for ModBanView {
  type DbTuple = ModBanViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        mod_ban: a.0.to_owned(),
        moderator: a.1.to_owned(),
        banned_user: a.2.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
