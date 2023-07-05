use crate::structs::CommunityBlockView;
use diesel::{result::Error, ExpressionMethods, QueryDsl};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{community, community_block, person},
  source::{community::Community, person::Person},
  traits::JoinView,
  utils::{GetConn, RunQueryDsl},
};

type CommunityBlockViewTuple = (Person, Community);

impl CommunityBlockView {
  pub async fn for_person(mut conn: impl GetConn, person_id: PersonId) -> Result<Vec<Self>, Error> {
    let res = community_block::table
      .inner_join(person::table)
      .inner_join(community::table)
      .select((person::all_columns, community::all_columns))
      .filter(community_block::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .order_by(community_block::published)
      .load::<CommunityBlockViewTuple>(conn)
      .await?;

    Ok(res.into_iter().map(Self::from_tuple).collect())
  }
}

impl JoinView for CommunityBlockView {
  type JoinTuple = CommunityBlockViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      person: a.0,
      community: a.1,
    }
  }
}
