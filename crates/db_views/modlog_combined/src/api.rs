use crate::ModlogCombinedView;
use lemmy_db_schema::{
  newtypes::{CommentId, CommunityId, PaginationCursor, PersonId, PostId},
  ModlogActionType,
};
use lemmy_db_schema_file::enums::ListingType;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Fetches the modlog.
pub struct GetModlog {
  /// Filter by the moderator.
  #[cfg_attr(feature = "full", ts(optional))]
  pub mod_person_id: Option<PersonId>,
  /// Filter by the community.
  #[cfg_attr(feature = "full", ts(optional))]
  pub community_id: Option<CommunityId>,
  /// Filter by the modlog action type.
  #[cfg_attr(feature = "full", ts(optional))]
  pub type_: Option<ModlogActionType>,
  /// Filter by listing type. When not using All, it will remove the non-community modlog entries,
  /// such as site bans, instance blocks, adding an admin, etc.
  #[cfg_attr(feature = "full", ts(optional))]
  pub listing_type: Option<ListingType>,
  /// Filter by the other / modded person.
  #[cfg_attr(feature = "full", ts(optional))]
  pub other_person_id: Option<PersonId>,
  /// Filter by post. Will include comments of that post.
  #[cfg_attr(feature = "full", ts(optional))]
  pub post_id: Option<PostId>,
  /// Filter by comment.
  #[cfg_attr(feature = "full", ts(optional))]
  pub comment_id: Option<CommentId>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_cursor: Option<PaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub page_back: Option<bool>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The modlog fetch response.
pub struct GetModlogResponse {
  pub modlog: Vec<ModlogCombinedView>,
  /// the pagination cursor to use to fetch the next page
  #[cfg_attr(feature = "full", ts(optional))]
  pub next_page: Option<PaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub prev_page: Option<PaginationCursor>,
}
