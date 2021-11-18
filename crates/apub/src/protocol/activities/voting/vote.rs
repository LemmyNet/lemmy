use crate::{fetcher::post_or_comment::PostOrComment, objects::person::ApubPerson};
use activitystreams::{primitives::OneOrMany, unparsed::Unparsed};
use anyhow::anyhow;
use lemmy_apub_lib::object_id::ObjectId;
use lemmy_utils::LemmyError;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use strum_macros::ToString;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vote {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) to: Option<OneOrMany<Url>>,
  pub(crate) object: ObjectId<PostOrComment>,
  pub(crate) cc: Option<OneOrMany<Url>>,
  #[serde(rename = "type")]
  pub(crate) kind: VoteType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}

#[derive(Clone, Debug, ToString, Deserialize, Serialize)]
pub enum VoteType {
  Like,
  Dislike,
}

impl TryFrom<i16> for VoteType {
  type Error = LemmyError;

  fn try_from(value: i16) -> Result<Self, Self::Error> {
    match value {
      1 => Ok(VoteType::Like),
      -1 => Ok(VoteType::Dislike),
      _ => Err(anyhow!("invalid vote value").into()),
    }
  }
}

impl From<&VoteType> for i16 {
  fn from(value: &VoteType) -> i16 {
    match value {
      VoteType::Like => 1,
      VoteType::Dislike => -1,
    }
  }
}
