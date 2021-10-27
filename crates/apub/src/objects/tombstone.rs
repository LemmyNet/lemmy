use crate::context::lemmy_context;
use activitystreams::{
  base::AnyBase,
  chrono::{DateTime, FixedOffset, NaiveDateTime},
  object::kind::TombstoneType,
  primitives::OneOrMany,
};
use lemmy_utils::utils::convert_datetime;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tombstone {
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(rename = "type")]
  kind: TombstoneType,
  former_type: String,
  deleted: DateTime<FixedOffset>,
}

impl Tombstone {
  pub fn new<T: ToString>(former_type: T, updated_time: NaiveDateTime) -> Tombstone {
    Tombstone {
      context: lemmy_context(),
      kind: TombstoneType::Tombstone,
      former_type: former_type.to_string(),
      deleted: convert_datetime(updated_time),
    }
  }
}
