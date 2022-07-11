// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::{
  objects::community::ApubCommunity,
  protocol::{activities::following::follow::FollowCommunity, Unparsed},
};
use activitypub_federation::core::object_id::ObjectId;
use activitystreams_kinds::activity::AcceptType;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptFollowCommunity {
  pub(crate) actor: ObjectId<ApubCommunity>,
  pub(crate) object: FollowCommunity,
  #[serde(rename = "type")]
  pub(crate) kind: AcceptType,
  pub(crate) id: Url,

  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
