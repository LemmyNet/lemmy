use crate::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{activities::voting::vote::Vote, InCommunity},
};
use activitypub_federation::{config::Data, fetch::object_id::ObjectId, kinds::activity::UndoType};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoVote {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) object: Vote,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
}

impl InCommunity for UndoVote {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let community = self.object.community(context).await?;
    Ok(community)
  }
}
