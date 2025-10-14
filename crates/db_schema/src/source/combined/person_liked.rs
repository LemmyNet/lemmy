use crate::newtypes::{CommentId, PersonId, PersonLikedCombinedId, PostId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::person_liked_combined;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = person_liked_combined))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = person_liked_combined_keys))]
/// A combined person_liked table.
pub struct PersonLikedCombined {
  pub voted_at: DateTime<Utc>,
  pub id: PersonLikedCombinedId,
  pub person_id: PersonId,
  pub post_id: Option<PostId>,
  pub comment_id: Option<CommentId>,
  pub vote_is_upvote: bool,
}
