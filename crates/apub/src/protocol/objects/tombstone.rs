use crate::protocol::Id;
use activitypub_federation::kinds::object::TombstoneType;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tombstone {
  pub(crate) id: Url,
  #[serde(rename = "type")]
  pub(crate) kind: TombstoneType,
}

impl Tombstone {
  pub fn new(id: Url) -> Tombstone {
    Tombstone {
      id,
      kind: TombstoneType::Tombstone,
    }
  }
}

impl Id for Tombstone {
  fn object_id(&self) -> &Url {
    &self.id
  }
}
