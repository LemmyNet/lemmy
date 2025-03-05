use crate::newtypes::{PersonId, PostKeywordBlockId};
#[cfg(feature = "full")]
use crate::schema::post_keyword_block;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable, TS)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::person::Person)))]
#[cfg_attr(feature = "full", diesel(table_name = post_keyword_block))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
pub struct PostKeywordBlock {
  pub id: PostKeywordBlockId,
  pub keyword: String,
  pub person_id: PersonId,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_keyword_block))]
pub struct PostKeywordBlockForm {
  pub person_id: PersonId,
  pub keyword: String,
}
