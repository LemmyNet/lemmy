use crate::{
  activity_lists::AnnouncableActivities,
  fetcher::object_id::ObjectId,
  objects::community::ApubCommunity,
};
use activitystreams::{activity::kind::AnnounceType, unparsed::Unparsed};
use lemmy_apub_lib::traits::ActivityFields;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, ActivityFields)]
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
