use crate::local_instance;
use activitypub_federation::{deser::values::MediaTypeMarkdown, utils::fetch_object_http};
use activitystreams_kinds::object::ImageType;
use lemmy_db_schema::newtypes::DbUrl;
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

pub mod activities;
pub(crate) mod collections;
pub(crate) mod objects;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
  pub(crate) content: String,
  pub(crate) media_type: MediaTypeMarkdown,
}

impl Source {
  pub(crate) fn new(content: String) -> Self {
    Source {
      content,
      media_type: MediaTypeMarkdown::Markdown,
    }
  }
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Unparsed(HashMap<String, serde_json::Value>);

pub(crate) trait Id {
  fn object_id(&self) -> &Url;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum IdOrNestedObject<Kind: Id> {
  Id(Url),
  NestedObject(Kind),
}

impl<Kind: Id + DeserializeOwned> IdOrNestedObject<Kind> {
  pub(crate) fn id(&self) -> &Url {
    match self {
      IdOrNestedObject::Id(i) => i,
      IdOrNestedObject::NestedObject(n) => n.object_id(),
    }
  }
  pub(crate) async fn object(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<Kind, LemmyError> {
    match self {
      IdOrNestedObject::Id(i) => {
        Ok(fetch_object_http(&i, local_instance(context), request_counter).await?)
      }
      IdOrNestedObject::NestedObject(o) => Ok(o),
    }
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use activitypub_federation::deser::context::WithContext;
  use assert_json_diff::assert_json_include;
  use lemmy_utils::error::LemmyError;
  use serde::{de::DeserializeOwned, Serialize};
  use std::{collections::HashMap, fs::File, io::BufReader};

  pub(crate) fn file_to_json_object<T: DeserializeOwned>(path: &str) -> Result<T, LemmyError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
  }

  pub(crate) fn test_json<T: DeserializeOwned>(path: &str) -> Result<WithContext<T>, LemmyError> {
    file_to_json_object::<WithContext<T>>(path)
  }

  /// Check that json deserialize -> serialize -> deserialize gives identical file as initial one.
  /// Ensures that there are no breaking changes in sent data.
  pub(crate) fn test_parse_lemmy_item<T: Serialize + DeserializeOwned + std::fmt::Debug>(
    path: &str,
  ) -> Result<T, LemmyError> {
    // parse file as T
    let parsed = file_to_json_object::<T>(path)?;

    // parse file into hashmap, which ensures that every field is included
    let raw = file_to_json_object::<HashMap<String, serde_json::Value>>(path)?;
    // assert that all fields are identical, otherwise print diff
    assert_json_include!(actual: &parsed, expected: raw);
    Ok(parsed)
  }
}
