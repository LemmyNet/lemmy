use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{community, mod_remove_community, person},
  source::{
    community::{Community, CommunitySafe},
    moderator::ModRemoveCommunity,
    person::{Person, PersonSafe},
  },
  PersonId,
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ModRemoveCommunityView {
  pub mod_remove_community: ModRemoveCommunity,
  pub moderator: PersonSafe,
  pub community: CommunitySafe,
}

type ModRemoveCommunityTuple = (ModRemoveCommunity, PersonSafe, CommunitySafe);

impl ModRemoveCommunityView {
  pub fn list(
    conn: &PgConnection,
    mod_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_remove_community::table
      .inner_join(person::table)
      .inner_join(community::table)
      .select((
        mod_remove_community::all_columns,
        Person::safe_columns_tuple(),
        Community::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(mod_person_id) = mod_person_id {
      query = query.filter(mod_remove_community::mod_person_id.eq(mod_person_id));
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
