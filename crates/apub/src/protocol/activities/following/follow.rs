use crate::{fetcher::user_or_community::UserOrCommunity, objects::person::ApubPerson};
use activitypub_federation::{
    fetch::object_id::ObjectId, kinds::activity::FollowType,
    protocol::helpers::deserialize_skip_error,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    pub(crate) actor: ObjectId<ApubPerson>,
    /// Optional, for compatibility with platforms that always expect recipient field
    #[serde(deserialize_with = "deserialize_skip_error", default)]
    pub(crate) to: Option<[ObjectId<UserOrCommunity>; 1]>,
    pub(crate) object: ObjectId<UserOrCommunity>,
    #[serde(rename = "type")]
    pub(crate) kind: FollowType,
    pub(crate) id: Url,
}
