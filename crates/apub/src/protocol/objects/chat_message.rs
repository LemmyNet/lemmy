use crate::{
  objects::{person::ApubPerson, private_message::ApubPrivateMessage},
  protocol::SourceCompat,
};
use chrono::{DateTime, FixedOffset};
use lemmy_apub_lib::{object_id::ObjectId, values::MediaTypeHtml};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
  pub(crate) r#type: ChatMessageType,
  pub(crate) id: ObjectId<ApubPrivateMessage>,
  pub(crate) attributed_to: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "crate::deserialize_one")]
  pub(crate) to: [ObjectId<ApubPerson>; 1],
  pub(crate) content: String,

  pub(crate) media_type: Option<MediaTypeHtml>,
  pub(crate) source: Option<SourceCompat>,
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
}

/// https://docs.pleroma.social/backend/development/ap_extensions/#chatmessages
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ChatMessageType {
  ChatMessage,
}
