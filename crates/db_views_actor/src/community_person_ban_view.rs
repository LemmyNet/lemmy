use crate::structs::CommunityPersonBanView;
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::{community, community_person_ban, person},
  source::{
    community::{Community, CommunitySafe},
    person::{Person, PersonSafe},
  },
  traits::ToSafe,
};

impl CommunityPersonBanView {
  pub fn get(
    conn: &mut PgConnection,
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
      .filter(
        community_person_ban::expires
          .is_null()
          .or(community_person_ban::expires.gt(now)),
      )
      .order_by(community_person_ban::published)
      .first::<(CommunitySafe, PersonSafe)>(conn)?;

    Ok(CommunityPersonBanView { community, person })
  }
}
