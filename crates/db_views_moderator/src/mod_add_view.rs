use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{mod_add, user_, user_alias_1},
  source::{
    moderator::ModAdd,
    user::{UserAlias1, UserSafe, UserSafeAlias1, User_},
  },
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ModAddView {
  pub mod_add: ModAdd,
  pub moderator: UserSafe,
  pub modded_user: UserSafeAlias1,
}

type ModAddViewTuple = (ModAdd, UserSafe, UserSafeAlias1);

impl ModAddView {
  pub fn list(
    conn: &PgConnection,
    mod_user_id: Option<i32>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_add::table
      .inner_join(user_::table.on(mod_add::mod_user_id.eq(user_::id)))
      .inner_join(user_alias_1::table.on(mod_add::other_user_id.eq(user_alias_1::id)))
      .select((
        mod_add::all_columns,
        User_::safe_columns_tuple(),
        UserAlias1::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(mod_user_id) = mod_user_id {
      query = query.filter(mod_add::mod_user_id.eq(mod_user_id));
    };

    let (limit, offset) = limit_and_offset(page, limit);

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_add::when_.desc())
      .load::<ModAddViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for ModAddView {
  type DbTuple = ModAddViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        mod_add: a.0.to_owned(),
        moderator: a.1.to_owned(),
        modded_user: a.2.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
