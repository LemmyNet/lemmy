use crate::structs::CommunityPersonBanView;
use diesel::{dsl::exists, result::Error, select, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::{community, community_person_ban, person},
  utils::{get_conn, DbPool},
};

impl CommunityPersonBanView {
  pub async fn get(
    pool: &mut DbPool<'_>,
    from_person_id: PersonId,
    from_community_id: CommunityId,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(
      community_person_ban::table
        .inner_join(community::table)
        .inner_join(person::table)
        .select((community::all_columns, person::all_columns))
        .filter(community_person_ban::community_id.eq(from_community_id))
        .filter(community_person_ban::person_id.eq(from_person_id))
        .order_by(community_person_ban::published),
    ))
    .get_result::<bool>(conn)
    .await
  }
}
