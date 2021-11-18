use crate::{activity_lists::AnnouncableActivities, objects::community::ApubCommunity};
use activitystreams::{activity::kind::AnnounceType, primitives::OneOrMany, unparsed::Unparsed};
use lemmy_apub_lib::object_id::ObjectId;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
  pub(crate) actor: ObjectId<ApubCommunity>,
  pub(crate) to: Option<OneOrMany<Url>>,
  pub(crate) object: AnnouncableActivities,
  pub(crate) cc: Option<OneOrMany<Url>>,
  #[serde(rename = "type")]
  pub(crate) kind: AnnounceType,
  pub(crate) id: Url,
  #[serde(flatten)]
  pub(crate) unparsed: Unparsed,
}
