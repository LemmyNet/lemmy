use lemmy_db_schema::{
  ModlogKindFilter,
  newtypes::{CommentId, CommunityId, PostId},
};
use lemmy_db_schema_file::{PersonId, enums::ListingType};
use lemmy_diesel_utils::pagination::PaginationCursor;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Fetches the modlog.
pub struct GetModlog {
  /// Filter by the moderator.
  pub mod_person_id: Option<PersonId>,
  /// Filter by the community.
  pub community_id: Option<CommunityId>,
  /// Filter by the modlog action type.
  pub type_: Option<ModlogKindFilter>,
  /// Filter by listing type. When not using All, it will remove the non-community modlog entries,
  /// such as site bans, instance blocks, adding an admin, etc.
  pub listing_type: Option<ListingType>,
  /// Filter by the other / modded person.
  pub other_person_id: Option<PersonId>,
  /// Filter by post. Will include comments of that post.
  pub post_id: Option<PostId>,
  /// Filter by comment.
  pub comment_id: Option<CommentId>,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}
