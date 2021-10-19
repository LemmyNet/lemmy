use crate::{
  schema::community_block,
  source::community::Community,
  CommunityBlockId,
  CommunityId,
  PersonId,
};
use serde::{Deserialize, Serialize};

#[derive(
  Clone, Queryable, Associations, Identifiable, PartialEq, Debug, Serialize, Deserialize,
)]
#[table_name = "community_block"]
#[belongs_to(Community)]
pub struct CommunityBlock {
  pub id: CommunityBlockId,
  pub person_id: PersonId,
  pub community_id: CommunityId,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "community_block"]
pub struct CommunityBlockForm {
  pub person_id: PersonId,
  pub community_id: CommunityId,
}
