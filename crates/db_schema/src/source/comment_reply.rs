use crate::newtypes::{CommentId, CommentReplyId, PersonId};
use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
use crate::schema::comment_reply;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::comment::Comment)))]
#[cfg_attr(feature = "full", diesel(table_name = comment_reply))]
/// This table keeps a list of replies to comments and posts.
pub struct CommentReply {
  pub id: CommentReplyId,
  pub recipient_id: PersonId,
  pub comment_id: CommentId,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = comment_reply))]
pub struct CommentReplyForm {
  pub recipient_id: PersonId,
  pub comment_id: CommentId,
  pub read: Option<bool>,
}
