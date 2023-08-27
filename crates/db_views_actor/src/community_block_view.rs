use crate::structs::CommunityBlockView;
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{community, community_block, person},
  source::{community::Community, person::Person},
  utils::{get_conn, DbPool},
};

impl CommunityBlockView {
  pub async fn for_person(pool: &mut DbPool<'_>, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    community_block::table
      .inner_join(person::table)
      .inner_join(community::table)
      .select((person::all_columns, community::all_columns))
      .filter(community_block::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .order_by(community_block::published)
      .load::<CommunityBlockView>(conn)
      .await
  }
}
