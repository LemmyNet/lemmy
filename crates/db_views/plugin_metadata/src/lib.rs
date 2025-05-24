use serde::{Deserialize, Serialize};
use url::Url;

#[cfg(feature = "full")]
use {
  extism::FromBytes,
  extism_convert::{encoding, Json},
  ts_rs::TS,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(TS, FromBytes))]
#[cfg_attr(feature = "full", ts(export))]
#[cfg_attr(feature = "full", encoding(Json))]
pub struct PluginMetadata {
  name: String,
  url: Url,
  description: String,
}
