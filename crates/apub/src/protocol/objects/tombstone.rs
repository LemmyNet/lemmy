// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::protocol::Id;
use activitystreams_kinds::object::TombstoneType;
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
  fn id(&self) -> &Url {
    &self.id
  }
}
