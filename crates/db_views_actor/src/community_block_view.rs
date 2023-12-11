use crate::structs::CommunityBlockView;
use diesel::{
  result::{Error, Error::QueryBuilderError},
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{community, community_block, person},
  utils::ActualDbPool,
};
use std::ops::DerefMut;

impl CommunityBlockView {
  pub async fn for_person(pool: &ActualDbPool, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let mut conn = pool.get().await.map_err(|e| QueryBuilderError(e.into()))?;
    community_block::table
      .inner_join(person::table)
      .inner_join(community::table)
      .select((person::all_columns, community::all_columns))
      .filter(community_block::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .order_by(community_block::published)
      .load::<CommunityBlockView>(conn.deref_mut())
      .await
  }
}
