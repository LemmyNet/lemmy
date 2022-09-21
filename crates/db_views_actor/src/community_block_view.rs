use crate::structs::CommunityBlockView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{community, community_block, person},
  source::{
    community::{Community, CommunitySafe},
    person::{Person, PersonSafe},
  },
  traits::{ToSafe, ViewToVec},
};

type CommunityBlockViewTuple = (PersonSafe, CommunitySafe);

impl CommunityBlockView {
  pub fn for_person(conn: &PgConnection, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let res = community_block::table
      .inner_join(person::table)
      .inner_join(community::table)
      .select((
        Person::safe_columns_tuple(),
        Community::safe_columns_tuple(),
      ))
      .filter(community_block::person_id.eq(person_id))
      .order_by(community_block::published)
      .load::<CommunityBlockViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for CommunityBlockView {
  type DbTuple = CommunityBlockViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        person: a.0,
        community: a.1,
      })
      .collect::<Vec<Self>>()
  }
}
