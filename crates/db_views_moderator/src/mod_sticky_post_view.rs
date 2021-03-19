use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{community, mod_sticky_post, person, post},
  source::{
    community::{Community, CommunitySafe},
    moderator::ModStickyPost,
    person::{Person, PersonSafe},
    post::Post,
  },
  CommunityId,
  PersonId,
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ModStickyPostView {
  pub mod_sticky_post: ModStickyPost,
  pub moderator: PersonSafe,
  pub post: Post,
  pub community: CommunitySafe,
}

type ModStickyPostViewTuple = (ModStickyPost, PersonSafe, Post, CommunitySafe);

impl ModStickyPostView {
  pub fn list(
    conn: &PgConnection,
    community_id: Option<CommunityId>,
    mod_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_sticky_post::table
      .inner_join(person::table)
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .select((
        mod_sticky_post::all_columns,
        Person::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(community_id) = community_id {
      query = query.filter(post::community_id.eq(community_id));
    };

    if let Some(mod_person_id) = mod_person_id {
      query = query.filter(mod_sticky_post::mod_person_id.eq(mod_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit);

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_sticky_post::when_.desc())
      .load::<ModStickyPostViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for ModStickyPostView {
  type DbTuple = ModStickyPostViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        mod_sticky_post: a.0.to_owned(),
        moderator: a.1.to_owned(),
        post: a.2.to_owned(),
        community: a.3.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
