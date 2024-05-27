use crate::newtypes::{CommunityId, PersonId};
#[cfg(feature = "full")]
use crate::schema::community_actions;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::{dsl, expression_methods::NullableExpressionMethods};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable)
)]
#[cfg_attr(
  feature = "full",
  diesel(belongs_to(crate::source::community::Community))
)]
#[cfg_attr(feature = "full", diesel(table_name = community_actions))]
#[cfg_attr(feature = "full", diesel(primary_key(person_id, community_id)))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct CommunityBlock {
  pub person_id: PersonId,
  pub community_id: CommunityId,
  #[diesel(select_expression = community_actions::blocked.assume_not_null())]
  #[diesel(select_expression_type = dsl::AssumeNotNull<community_actions::blocked>)]
  pub published: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = community_actions))]
pub struct CommunityBlockForm {
  pub person_id: PersonId,
  pub community_id: CommunityId,
}
