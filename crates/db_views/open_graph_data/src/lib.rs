use lemmy_db_schema::newtypes::DbUrl;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Default, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
/// Site metadata, from its opengraph tags.
pub struct OpenGraphData {
  #[cfg_attr(feature = "full", ts(optional))]
  pub title: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub description: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub image: Option<DbUrl>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub embed_video_url: Option<DbUrl>,
}
