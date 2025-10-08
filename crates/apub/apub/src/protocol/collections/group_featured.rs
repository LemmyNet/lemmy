use activitypub_federation::kinds::collection::OrderedCollectionType;
use lemmy_apub_objects::protocol::page::Page;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupFeatured {
  pub(crate) r#type: OrderedCollectionType,
  pub(crate) id: Url,
  pub(crate) total_items: i64,
  pub(crate) ordered_items: Vec<Page>,
}
