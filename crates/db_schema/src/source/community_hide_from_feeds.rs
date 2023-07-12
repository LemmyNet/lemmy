use crate::newtypes::{CommunityHideFromFeedsId, CommunityId, PersonId};
#[cfg(feature = "full")]
use crate::schema::community_hide_from_feeds;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::community::Community))
)]
#[cfg_attr(feature = "full", diesel(table_name = community_hide_from_feeds))]
pub struct CommunityHideFromFeeds {
  pub id: CommunityHideFromFeedsId,
  pub person_id: PersonId,
  pub community_id: CommunityId,
  pub published: chrono::NaiveDateTime,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_hide_from_feeds))]
pub struct CommunityHideFromFeedsForm {
  pub person_id: PersonId,
  pub community_id: CommunityId,
}
