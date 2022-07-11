// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

use crate::{
  objects::{person::ApubPerson, private_message::ApubPrivateMessage},
  protocol::Source,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  deser::{
    helpers::{deserialize_one, deserialize_skip_error},
    values::MediaTypeHtml,
  },
};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
  pub(crate) r#type: ChatMessageType,
  pub(crate) id: ObjectId<ApubPrivateMessage>,
  pub(crate) attributed_to: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one")]
  pub(crate) to: [ObjectId<ApubPerson>; 1],
  pub(crate) content: String,

  pub(crate) media_type: Option<MediaTypeHtml>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) source: Option<Source>,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
}

/// https://docs.pleroma.social/backend/development/ap_extensions/#chatmessages
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ChatMessageType {
  ChatMessage,
}
