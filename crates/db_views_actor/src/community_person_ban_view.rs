use crate::structs::CommunityPersonBanView;
use diesel::{dsl::now, result::Error, BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::{community, community_person_ban, person},
  source::{
    community::{Community, CommunitySafe},
    person::{Person, PersonSafe},
  },
  traits::ToSafe,
  utils::{get_conn, DbPool},
};

impl CommunityPersonBanView {
  pub async fn get(
    pool: &DbPool,
    from_person_id: PersonId,
    from_community_id: CommunityId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
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
      .first::<(CommunitySafe, PersonSafe)>(conn)
      .await?;

    Ok(CommunityPersonBanView { community, person })
  }
}
