use activitypub_federation::kinds::collection::OrderedCollectionType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UrlCollection {
  pub(crate) r#type: OrderedCollectionType,
  pub(crate) id: String,
  pub(crate) total_items: i32,
  pub(crate) ordered_items: Vec<Url>,
}
