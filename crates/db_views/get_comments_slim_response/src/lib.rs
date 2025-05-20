use lemmy_db_schema::newtypes::PaginationCursor;
use lemmy_db_views_comment::CommentSlimView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A slimmer comment list response, without the post or community.
pub struct GetCommentsSlimResponse {
  pub comments: Vec<CommentSlimView>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub next_page: Option<PaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub prev_page: Option<PaginationCursor>,
}
