use lemmy_db_views_open_graph_data::OpenGraphData;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Default, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Site metadata, from its opengraph tags.
pub struct LinkMetadata {
  #[serde(flatten)]
  pub opengraph_data: OpenGraphData,
  #[cfg_attr(feature = "full", ts(optional))]
  pub content_type: Option<String>,
}
