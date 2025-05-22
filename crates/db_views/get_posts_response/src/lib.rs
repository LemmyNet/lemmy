use lemmy_db_schema::newtypes::PaginationCursor;
use lemmy_db_views_post::PostView;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The post list response.
pub struct GetPostsResponse {
  pub posts: Vec<PostView>,
  /// the pagination cursor to use to fetch the next page
  #[cfg_attr(feature = "full", ts(optional))]
  pub next_page: Option<PaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub prev_page: Option<PaginationCursor>,
}
