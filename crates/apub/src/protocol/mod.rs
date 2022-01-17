use activitystreams_kinds::object::ImageType;
use serde::{Deserialize, Serialize};
use url::Url;

use lemmy_apub_lib::values::MediaTypeMarkdown;
use lemmy_db_schema::newtypes::DbUrl;
use std::collections::HashMap;

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
  kind: ImageType,
  pub(crate) url: Url,
}

impl ImageObject {
  pub(crate) fn new(url: DbUrl) -> Self {
    ImageObject {
      kind: ImageType::Image,
      url: url.into(),
    }
  }
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Unparsed(HashMap<String, serde_json::Value>);

#[cfg(test)]
pub(crate) mod tests {
  use crate::objects::tests::file_to_json_object;
  use assert_json_diff::assert_json_include;
  use serde::{de::DeserializeOwned, Serialize};
  use std::collections::HashMap;

  /// Check that json deserialize -> serialize -> deserialize gives identical file as initial one.
  /// Ensures that there are no breaking changes in sent data.
  pub(crate) fn test_parse_lemmy_item<T: Serialize + DeserializeOwned + std::fmt::Debug>(
    path: &str,
  ) -> T {
    // parse file as T
    let parsed = file_to_json_object::<T>(path).unwrap();

    // parse file into hashmap, which ensures that every field is included
    let raw = file_to_json_object::<HashMap<String, serde_json::Value>>(path).unwrap();
    // assert that all fields are identical, otherwise print diff
    assert_json_include!(actual: &parsed, expected: raw);
    parsed
  }
}
