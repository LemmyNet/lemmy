use crate::newtypes::{CommentId, PersonId, PersonMentionId};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
use crate::schema::person_mention;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", belongs_to(crate::source::comment::Comment))]
#[cfg_attr(feature = "full", table_name = "person_mention")]
pub struct PersonMention {
  pub id: PersonMentionId,
  pub recipient_id: PersonId,
  pub comment_id: CommentId,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", table_name = "person_mention")]
pub struct PersonMentionForm {
  pub recipient_id: PersonId,
  pub comment_id: CommentId,
  pub read: Option<bool>,
}
