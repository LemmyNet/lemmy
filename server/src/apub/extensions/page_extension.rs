use activitystreams::{ext::Extension, Base};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageExtension {
  pub comments_enabled: bool,
  pub sensitive: bool,
}

impl<T> Extension<T> for PageExtension where T: Base {}
