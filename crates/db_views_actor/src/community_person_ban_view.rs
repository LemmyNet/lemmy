use crate::structs::CommunityPersonBanView;
use diesel::{dsl::now, result::Error, BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::{community, community_person_ban, person},
  source::{community::Community, person::Person},
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
      .select((community::all_columns, person::all_columns))
      .filter(community_person_ban::community_id.eq(from_community_id))
      .filter(community_person_ban::person_id.eq(from_person_id))
      .filter(
        community_person_ban::expires
          .is_null()
          .or(community_person_ban::expires.gt(now)),
      )
      .order_by(community_person_ban::published)
      .first::<(Community, Person)>(conn)
      .await?;

    Ok(CommunityPersonBanView { community, person })
  }
}
