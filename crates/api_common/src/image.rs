use lemmy_db_schema::newtypes::CommunityId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use ts_rs::TS;
use url::Url;

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ImageGetParams {
  #[cfg_attr(feature = "full", ts(optional))]
  pub file_type: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub max_size: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct DeleteImageParams {
  pub filename: String,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct ImageProxyParams {
  pub url: String,
  #[cfg_attr(feature = "full", ts(optional))]
  pub file_type: Option<String>,
  #[cfg_attr(feature = "full", ts(optional))]
  pub max_size: Option<i32>,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct UploadImageResponse {
  pub image_url: Url,
  pub filename: String,
}

/// Parameter for setting community icon or banner. Can't use POST data here as it already contains
/// the image data.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "full", derive(TS))]
#[cfg_attr(feature = "full", ts(export))]
pub struct CommunityIdQuery {
  pub id: CommunityId,
}
