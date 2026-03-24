use activitypub_federation::{
  fetch::object_id::ObjectId,
  kinds::collection::OrderedCollectionType,
};
use crate::objects::person::ApubPerson;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedModerators {
  pub(crate) r#type: OrderedCollectionType,
  pub(crate) id: Url,
  pub(crate) ordered_items: Vec<ObjectId<ApubPerson>>,
}
