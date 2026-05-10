use crate::newtypes::{CommentId, CommunityId, PersonContentCombinedId, PostId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
use lemmy_db_schema_file::PersonId;
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
  pub creator_id: PersonId,
  pub post_id: PostId,
  pub comment_id: Option<CommentId>,
  pub id: PersonContentCombinedId,
  pub community_id: CommunityId,
}
