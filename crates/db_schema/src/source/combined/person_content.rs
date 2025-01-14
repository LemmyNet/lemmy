use crate::newtypes::{CommentId, PersonContentCombinedId, PostId};
#[cfg(feature = "full")]
use crate::schema::person_content_combined;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;

#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = person_content_combined))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = person_content_combined_keys))]
/// A combined table for a persons contents (posts and comments)
pub struct PersonContentCombined {
  pub id: PersonContentCombinedId,
  pub published: DateTime<Utc>,
  pub post_id: Option<PostId>,
  pub comment_id: Option<CommentId>,
}
