use crate::newtypes::{CommentId, PersonId, PersonSavedCombinedId, PostId};
#[cfg(feature = "full")]
use crate::schema::person_saved_combined;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use i_love_jesus::CursorKeysModule;

#[derive(PartialEq, Eq, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, CursorKeysModule)
)]
#[cfg_attr(feature = "full", diesel(table_name = person_saved_combined))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "full", cursor_keys_module(name = person_saved_combined_keys))]
/// A combined person_saved table.
pub struct PersonSavedCombined {
  pub id: PersonSavedCombinedId,
  pub saved: DateTime<Utc>,
  pub person_id: PersonId,
  pub post_id: Option<PostId>,
  pub comment_id: Option<CommentId>,
}
