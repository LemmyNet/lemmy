use crate::newtypes::{CommentId, PersonId, PersonMentionId, PostId};
#[cfg(feature = "full")]
use crate::schema::person_mention;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::comment::Comment)))]
#[cfg_attr(feature = "full", diesel(table_name = person_mention))]
pub struct PersonMention {
  pub id: PersonMentionId,
  pub recipient_id: PersonId,
  pub comment_id: Option<CommentId>,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
  pub post_id: Option<PostId>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_mention))]
pub struct PersonMentionInsertForm {
  pub recipient_id: PersonId,
  pub comment_id: Option<CommentId>,
  pub post_id: Option<PostId>,
  pub read: Option<bool>,
}

#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_mention))]
pub struct PersonMentionUpdateForm {
  pub read: Option<bool>,
}
