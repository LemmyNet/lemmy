use crate::{
  objects::{person::ApubPerson, private_message::ApubPrivateMessage},
  protocol::Source,
};
use activitypub_federation::{
  fetch::object_id::ObjectId,
  protocol::{
    helpers::{deserialize_one, deserialize_skip_error},
    values::MediaTypeHtml,
  },
};
use activitypub_federation::kinds::object::NoteType;
use activitypub_federation::kinds::public;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error;
use serde_with::skip_serializing_none;
use url::Url;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
  pub(crate) r#type: NoteType,
  pub(crate) id: ObjectId<ApubPrivateMessage>,
  pub(crate) attributed_to: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_not_public")]
  pub(crate) to: [ObjectId<ApubPerson>; 1],
  pub(crate) content: String,

  pub(crate) media_type: Option<MediaTypeHtml>,
  #[serde(deserialize_with = "deserialize_skip_error", default)]
  pub(crate) source: Option<Source>,
  /// cc field is unused, but we nned to check that it doesnt
  /// todo: need to turn this into option with default
  //#[serde(deserialize_with = "deserialize_one_not_public", skip_serializing)]
  //pub(crate) cc: [ObjectId<ApubPerson>; 1],
  pub(crate) published: Option<DateTime<FixedOffset>>,
  pub(crate) updated: Option<DateTime<FixedOffset>>,
}

/// Only allows deserialization if the field is missing or null. If it is present, throws an error.
pub fn deserialize_one_not_public<'de, T, D>(deserializer: D) -> Result<[T;1], D::Error>
  where
      D: Deserializer<'de>,
      T: Deserialize<'de> + Into<Url> + Clone,
{
  let d: [T;1] = deserialize_one(deserializer)?;
  let url = d[0].clone().into();
  if url == public() {
    return Err(D::Error::custom("Private message must not have `public` in to or cc"));
  }
  Ok(d)

}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

use super::*;
  use crate::protocol::objects::note::Note;
  use crate::protocol::tests::test_json;

  #[test]
  fn deserialize_private_message() {
    assert!(test_json::<ChatMessage>("assets/mastodon/objects/private_message.json").is_ok());
    assert!(test_json::<Note>("assets/mastodon/objects/private_message.json").is_err());
  }
}