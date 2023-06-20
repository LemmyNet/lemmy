use crate::newtypes::{CommunityId, CommunityMuteId, PersonId};
#[cfg(feature = "full")]
use crate::schema::community_mute;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::community::Community))
)]
#[cfg_attr(feature = "full", diesel(table_name = community_mute))]
pub struct CommunityMute {
  pub id: CommunityMuteId,
  pub person_id: PersonId,
  pub community_id: CommunityId,
  pub published: chrono::NaiveDateTime,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_mute))]
pub struct CommunityMuteForm {
  pub person_id: PersonId,
  pub community_id: CommunityId,
}
