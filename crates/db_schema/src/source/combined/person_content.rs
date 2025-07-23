use crate::newtypes::{CommentId, PersonContentCombinedId, PostId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::person_content_combined;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = person_content_combined))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = person_content_combined_keys))]
/// A combined table for a persons contents (posts and comments)
pub struct PersonContentCombined {
  pub published_at: DateTime<Utc>,
  pub post_id: Option<PostId>,
  pub comment_id: Option<CommentId>,
  pub id: PersonContentCombinedId,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_content_combined))]
pub struct PersonContentCombinedPostInsertForm {
  pub post_id: PostId,
  pub published_at: DateTime<Utc>,
}

#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = person_content_combined))]
pub struct PersonContentCombinedCommentInsertForm {
  pub comment_id: CommentId,
  pub published_at: DateTime<Utc>,
}
