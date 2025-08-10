use crate::newtypes::{CommentId, PersonId, PersonLikedCombinedId, PostId};
#[cfg(feature = "full")]
use crate::utils::queryable::BoolToIntScore;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::person_liked_combined;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = person_liked_combined))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// A combined person_liked table.
pub struct PersonLikedCombined {
  pub liked_at: DateTime<Utc>,
  #[cfg_attr(feature = "full", diesel(deserialize_as = BoolToIntScore))]
  #[cfg_attr(feature = "full", diesel(column_name = like_score_is_positive))]
  pub like_score: i16,
  pub person_id: PersonId,
  pub post_id: Option<PostId>,
  pub comment_id: Option<CommentId>,
  pub id: PersonLikedCombinedId,
}

#[cfg(feature = "full")]
#[derive(Queryable, Selectable, CursorKeysModule)]
#[diesel(table_name = person_liked_combined)]
#[cursor_keys_module(name = person_liked_combined_keys)]
pub struct PersonLikedCombinedCursorData {
  pub liked_at: DateTime<Utc>,
  pub id: PersonLikedCombinedId,
}
