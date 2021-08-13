use diesel::{result::Error, *};
use lemmy_db_queries::{ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{community, community_moderator, person},
  source::{
    community::{Community, CommunitySafe},
    person::{Person, PersonSafe},
  },
  CommunityId,
  PersonId,
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CommunityModeratorView {
  pub community: CommunitySafe,
  pub moderator: PersonSafe,
}

type CommunityModeratorViewTuple = (CommunitySafe, PersonSafe);

impl CommunityModeratorView {
  pub fn for_community(conn: &PgConnection, community_id: CommunityId) -> Result<Vec<Self>, Error> {
    let res = community_moderator::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
      ))
      .filter(community_moderator::community_id.eq(community_id))
      .order_by(community_moderator::published)
      .load::<CommunityModeratorViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }

  pub fn for_person(conn: &PgConnection, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let res = community_moderator::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
      ))
      .filter(community_moderator::person_id.eq(person_id))
      .order_by(community_moderator::published)
      .load::<CommunityModeratorViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }

  /// Finds all communities first mods / creators
  /// Ideally this should be a group by, but diesel doesn't support it yet
  pub fn get_community_first_mods(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    let res = community_moderator::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
      ))
      // A hacky workaround instead of group_bys
      // https://stackoverflow.com/questions/24042359/how-to-join-only-one-row-in-joined-table-with-postgres
      .distinct_on(community_moderator::community_id)
      .order_by((
        community_moderator::community_id,
        community_moderator::person_id,
      ))
      .load::<CommunityModeratorViewTuple>(conn)?;

    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for CommunityModeratorView {
  type DbTuple = CommunityModeratorViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        community: a.0.to_owned(),
        moderator: a.1.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
