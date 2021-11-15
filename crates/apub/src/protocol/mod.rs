use activitystreams::object::kind::ImageType;
use serde::{Deserialize, Serialize};
use url::Url;

use lemmy_apub_lib::values::MediaTypeMarkdown;

pub mod activities;
pub(crate) mod collections;
pub(crate) mod objects;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
  pub(crate) content: String,
  pub(crate) media_type: MediaTypeMarkdown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageObject {
  #[serde(rename = "type")]
  pub(crate) kind: ImageType,
  pub(crate) url: Url,
}

#[cfg(test)]
pub(crate) mod tests {
  use crate::objects::tests::file_to_json_object;
  use assert_json_diff::assert_json_include;
  use serde::{de::DeserializeOwned, Serialize};
  use std::collections::HashMap;

  pub(crate) fn test_parse_lemmy_item<T: Serialize + DeserializeOwned + std::fmt::Debug>(
    path: &str,
  ) -> T {
    let parsed = file_to_json_object::<T>(path);

    // ensure that no field is ignored when parsing
    let raw = file_to_json_object::<HashMap<String, serde_json::Value>>(path);
    assert_json_include!(actual: &parsed, expected: raw);
    parsed
  }
}
