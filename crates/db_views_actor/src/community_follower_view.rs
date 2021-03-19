use diesel::{result::Error, *};
use lemmy_db_queries::{ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{community, community_follower, person},
  source::{
    community::{Community, CommunitySafe},
    person::{Person, PersonSafe},
  },
  CommunityId,
  PersonId,
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CommunityFollowerView {
  pub community: CommunitySafe,
  pub follower: PersonSafe,
}

type CommunityFollowerViewTuple = (CommunitySafe, PersonSafe);

impl CommunityFollowerView {
  pub fn for_community(conn: &PgConnection, community_id: CommunityId) -> Result<Vec<Self>, Error> {
    let res = community_follower::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
      ))
      .filter(community_follower::community_id.eq(community_id))
      .order_by(community_follower::published)
      .load::<CommunityFollowerViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }

  pub fn for_person(conn: &PgConnection, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let res = community_follower::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
      ))
      .filter(community_follower::person_id.eq(person_id))
      .order_by(community_follower::published)
      .load::<CommunityFollowerViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for CommunityFollowerView {
  type DbTuple = CommunityFollowerViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        community: a.0.to_owned(),
        follower: a.1.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
