use crate::newtypes::{CommentId, PersonId, PersonMentionId};
#[cfg(feature = "full")]
use crate::schema::person_mention;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable, TS))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::comment::Comment)))]
#[cfg_attr(feature = "full", diesel(table_name = person_mention))]
#[cfg_attr(feature = "full", ts(export))]
/// A person mention.
pub struct PersonMention {
  pub id: PersonMentionId,
  pub recipient_id: PersonId,
  pub comment_id: CommentId,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_mention))]
pub struct PersonMentionInsertForm {
  pub recipient_id: PersonId,
  pub comment_id: CommentId,
  pub read: Option<bool>,
}

#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_mention))]
pub struct PersonMentionUpdateForm {
  pub read: Option<bool>,
}
