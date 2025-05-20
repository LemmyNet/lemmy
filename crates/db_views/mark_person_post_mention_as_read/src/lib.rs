use lemmy_db_schema::newtypes::PersonPostMentionId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Mark a person mention as read.
pub struct MarkPersonPostMentionAsRead {
  pub person_post_mention_id: PersonPostMentionId,
  pub read: bool,
}
