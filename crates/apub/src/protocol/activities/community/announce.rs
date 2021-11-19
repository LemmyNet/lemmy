use crate::{
  activity_lists::AnnouncableActivities,
  objects::community::ApubCommunity,
  protocol::Unparsed,
};
use activitystreams_kinds::activity::AnnounceType;
use lemmy_apub_lib::object_id::ObjectId;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
  pub(crate) actor: ObjectId<ApubCommunity>,
  pub(crate) to: Vec<Url>,
  pub(crate) object: AnnouncableActivities,
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: AnnounceType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
