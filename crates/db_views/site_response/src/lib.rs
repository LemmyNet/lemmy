use lemmy_db_views_site::SiteView;
use serde::{Deserialize, Serialize};
#[cfg(feature = "full")]
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// The response for a site.
pub struct SiteResponse {
  pub site_view: SiteView,
  /// deprecated, use field `tagline` or /api/v4/tagline/list
  pub taglines: Vec<()>,
}
