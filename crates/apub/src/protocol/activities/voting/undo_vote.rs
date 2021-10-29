use activitystreams::{activity::kind::UndoType, unparsed::Unparsed};
use serde::{Deserialize, Serialize};
use url::Url;

use lemmy_apub_lib::traits::ActivityFields;

use crate::{
  fetcher::object_id::ObjectId,
  objects::person::ApubPerson,
  protocol::activities::voting::vote::Vote,
};

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
#[serde(rename_all = "camelCase")]
pub struct UndoVote {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) to: Vec<Url>,
  pub(crate) object: Vote,
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
