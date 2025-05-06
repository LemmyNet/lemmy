
  use activitypub_federation::protocol::context::WithContext;
  use assert_json_diff::assert_json_include;
  use lemmy_utils::error::LemmyResult;
  use serde::{de::DeserializeOwned, Serialize};
  use std::{collections::HashMap, fs::File, io::BufReader};

  pub(crate) fn file_to_json_object<T: DeserializeOwned>(path: &str) -> LemmyResult<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
  }

  pub(crate) fn test_json<T: DeserializeOwned>(path: &str) -> LemmyResult<WithContext<T>> {
    file_to_json_object::<WithContext<T>>(path)
  }

  /// Check that json deserialize -> serialize -> deserialize gives identical file as initial one.
  /// Ensures that there are no breaking changes in sent data.
  pub(crate) fn test_parse_lemmy_item<T: Serialize + DeserializeOwned + std::fmt::Debug>(
    path: &str,
  ) -> LemmyResult<T> {
    // parse file as T
    let parsed = file_to_json_object::<T>(path)?;

    // parse file into hashmap, which ensures that every field is included
    let raw = file_to_json_object::<HashMap<String, serde_json::Value>>(path)?;
    // assert that all fields are identical, otherwise print diff
    assert_json_include!(actual: &parsed, expected: raw);
    Ok(parsed)
  }