use crate::newtypes::{PersonId};
#[cfg(feature = "full")]
use crate::schema::user_post_keyword_block;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, TS)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::person::Person)))]
#[cfg_attr(feature = "full", diesel(table_name = user_post_keyword_block))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct UserPostKeywordBlock {
  pub keyword: String,
  pub person_id: PersonId,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = user_post_keyword_block))]
pub struct UserPostKeywordBlockForm {
  pub person_id: PersonId,
  pub keyword: String,
}
