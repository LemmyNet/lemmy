use crate::newtypes::{CommentId, PersonId, PersonSavedCombinedId, PostId};
#[cfg(feature = "full")]
use crate::schema::person_saved_combined;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, TS, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = person_saved_combined))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", ts(export))]
#[cfg_attr(feature = "full", cursor_keys_module(name = person_saved_combined_keys))]
/// A combined person_saved table.
pub struct PersonSavedCombined {
  pub id: PersonSavedCombinedId,
  pub published: DateTime<Utc>,
  pub person_id: PersonId,
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_id: Option<PostId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_id: Option<CommentId>,
}
