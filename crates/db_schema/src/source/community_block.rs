use crate::newtypes::{CommunityBlockId, CommunityId, PersonId};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
use crate::schema::community_block;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", belongs_to(crate::source::community::Community))]
#[cfg_attr(feature = "full", table_name = "community_block")]
pub struct CommunityBlock {
  pub id: CommunityBlockId,
  pub person_id: PersonId,
  pub community_id: CommunityId,
  pub published: chrono::NaiveDateTime,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "community_block")]
pub struct CommunityBlockForm {
  pub person_id: PersonId,
  pub community_id: CommunityId,
}
