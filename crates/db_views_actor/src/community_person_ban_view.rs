use diesel::{result::Error, *};
use lemmy_db_queries::ToSafe;
use lemmy_db_schema::{
  schema::{community, community_person_ban, person},
  source::{
    community::{Community, CommunitySafe},
    person::{Person, PersonSafe},
  },
  CommunityId,
  PersonId,
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct CommunityPersonBanView {
  pub community: CommunitySafe,
  pub person: PersonSafe,
}

impl CommunityPersonBanView {
  pub fn get(
    conn: &PgConnection,
    from_person_id: PersonId,
    from_community_id: CommunityId,
  ) -> Result<Self, Error> {
    let (community, person) = community_person_ban::table
      .inner_join(community::table)
      .inner_join(person::table)
      .select((
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
      ))
      .filter(community_person_ban::community_id.eq(from_community_id))
      .filter(community_person_ban::person_id.eq(from_person_id))
      .order_by(community_person_ban::published)
      .first::<(CommunitySafe, PersonSafe)>(conn)?;

    Ok(CommunityPersonBanView { community, person })
  }
}
