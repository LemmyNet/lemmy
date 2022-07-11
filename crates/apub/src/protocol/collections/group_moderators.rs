// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::objects::person::ApubPerson;
use activitypub_federation::core::object_id::ObjectId;
use activitystreams_kinds::collection::OrderedCollectionType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupModerators {
  pub(crate) r#type: OrderedCollectionType,
  pub(crate) id: Url,
  pub(crate) ordered_items: Vec<ObjectId<ApubPerson>>,
}
