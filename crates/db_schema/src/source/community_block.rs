use crate::newtypes::{CommunityBlockId, CommunityId, PersonId};
#[cfg(feature = "full")]
use crate::schema::community_block;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(
    feature = "full",
    diesel(belongs_to(crate::source::community::Community))
)]
#[cfg_attr(feature = "full", diesel(table_name = community_block))]
pub struct CommunityBlock {
    pub id: CommunityBlockId,
    pub person_id: PersonId,
    pub community_id: CommunityId,
    pub published: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_block))]
pub struct CommunityBlockForm {
    pub person_id: PersonId,
    pub community_id: CommunityId,
}
