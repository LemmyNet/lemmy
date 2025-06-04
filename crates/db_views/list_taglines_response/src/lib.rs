use lemmy_db_schema::{newtypes::PaginationCursor, source::tagline::Tagline};
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// A response for taglines.
pub struct ListTaglinesResponse {
  pub taglines: Vec<Tagline>,
  /// the pagination cursor to use to fetch the next page
  #[cfg_attr(feature = "full", ts(optional))]
  pub next_page: Option<PaginationCursor>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub prev_page: Option<PaginationCursor>,
}
