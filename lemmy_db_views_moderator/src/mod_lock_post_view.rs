use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{community, mod_lock_post, post, user_},
  source::{
    community::{Community, CommunitySafe},
    moderator::ModLockPost,
    post::Post,
    user::{UserSafe, User_},
  },
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ModLockPostView {
  pub mod_lock_post: ModLockPost,
  pub moderator: UserSafe,
  pub post: Post,
  pub community: CommunitySafe,
}

type ModLockPostViewTuple = (ModLockPost, UserSafe, Post, CommunitySafe);

impl ModLockPostView {
  pub fn list(
    conn: &PgConnection,
    community_id: Option<i32>,
    mod_user_id: Option<i32>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_lock_post::table
      .inner_join(user_::table)
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .select((
        mod_lock_post::all_columns,
        User_::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(community_id) = community_id {
      query = query.filter(post::community_id.eq(community_id));
    };

    if let Some(mod_user_id) = mod_user_id {
      query = query.filter(mod_lock_post::mod_user_id.eq(mod_user_id));
    };

    let (limit, offset) = limit_and_offset(page, limit);

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_lock_post::when_.desc())
      .load::<ModLockPostViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for ModLockPostView {
  type DbTuple = ModLockPostViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        mod_lock_post: a.0.to_owned(),
        moderator: a.1.to_owned(),
        post: a.2.to_owned(),
        community: a.3.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
