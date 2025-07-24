use activitypub_federation::kinds::collection::CollectionType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GroupFollowers {
  pub(crate) id: Url,
  pub(crate) r#type: CollectionType,
  pub(crate) total_items: i32,
  pub(crate) items: Vec<()>,
}
