use crate::newtypes::{PersonId, PersonPostMentionId, PostId};
#[cfg(feature = "full")]
use crate::schema::person_post_mention;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(
  feature = "full",
  derive(Queryable, Selectable, Associations, Identifiable, TS)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(table_name = person_post_mention))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
/// A person mention.
pub struct PersonPostMention {
  pub id: PersonPostMentionId,
  pub recipient_id: PersonId,
  pub post_id: PostId,
  pub read: bool,
  pub published: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_post_mention))]
pub struct PersonPostMentionInsertForm {
  pub recipient_id: PersonId,
  pub post_id: PostId,
  pub read: Option<bool>,
}

#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_post_mention))]
pub struct PersonPostMentionUpdateForm {
  pub read: Option<bool>,
}
