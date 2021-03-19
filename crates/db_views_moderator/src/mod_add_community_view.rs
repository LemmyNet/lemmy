use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{community, mod_add_community, person, person_alias_1},
  source::{
    community::{Community, CommunitySafe},
    moderator::ModAddCommunity,
    person::{Person, PersonAlias1, PersonSafe, PersonSafeAlias1},
  },
  CommunityId,
  PersonId,
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ModAddCommunityView {
  pub mod_add_community: ModAddCommunity,
  pub moderator: PersonSafe,
  pub community: CommunitySafe,
  pub modded_person: PersonSafeAlias1,
}

type ModAddCommunityViewTuple = (ModAddCommunity, PersonSafe, CommunitySafe, PersonSafeAlias1);

impl ModAddCommunityView {
  pub fn list(
    conn: &PgConnection,
    community_id: Option<CommunityId>,
    mod_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_add_community::table
      .inner_join(person::table.on(mod_add_community::mod_person_id.eq(person::id)))
      .inner_join(community::table)
      .inner_join(
        person_alias_1::table.on(mod_add_community::other_person_id.eq(person_alias_1::id)),
      )
      .select((
        mod_add_community::all_columns,
        Person::safe_columns_tuple(),
        Community::safe_columns_tuple(),
        PersonAlias1::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(mod_person_id) = mod_person_id {
      query = query.filter(mod_add_community::mod_person_id.eq(mod_person_id));
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
        modded_person: a.3.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
