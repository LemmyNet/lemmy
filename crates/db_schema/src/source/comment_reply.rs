use crate::newtypes::{CommentId, CommentReplyId, PersonId};
#[cfg(feature = "full")]
use crate::schema::comment_reply;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Associations, Identifiable, TS, WithoutId!))]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::comment::Comment)))]
#[cfg_attr(feature = "full", diesel(table_name = comment_reply))]
#[cfg_attr(feature = "full", ts(export))]
/// A comment reply.
pub struct CommentReply {
  pub id: CommentReplyId,
  pub recipient_id: PersonId,
  pub comment_id: CommentId,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = comment_reply))]
pub struct CommentReplyInsertForm {
  pub recipient_id: PersonId,
  pub comment_id: CommentId,
  pub read: Option<bool>,
}

#[cfg_attr(feature = "full", derive(AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = comment_reply))]
pub struct CommentReplyUpdateForm {
  pub read: Option<bool>,
}
