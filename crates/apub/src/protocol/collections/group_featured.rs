use crate::protocol::objects::page::Page;
use activitypub_federation::kinds::collection::OrderedCollectionType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupFeatured {
    pub(crate) r#type: OrderedCollectionType,
    pub(crate) id: Url,
    pub(crate) total_items: i32,
    pub(crate) ordered_items: Vec<Page>,
}
