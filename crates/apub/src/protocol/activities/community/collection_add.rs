use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::AddType,
  protocol::helpers::deserialize_one_or_many,
};
use itertools::Itertools;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, person::ApubPerson},
  utils::{functions::community_from_objects, protocol::InCommunity},
};
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionAdd {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: Url,
  pub(crate) target: Url,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: AddType,
  pub(crate) id: Url,
}

impl InCommunity for CollectionAdd {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let iter = self.to.iter().merge(self.cc.iter());

    community_from_objects(iter, context).await
  }
}
