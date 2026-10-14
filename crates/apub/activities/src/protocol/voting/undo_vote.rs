use super::vote::Vote;
use crate::protocol::IdOrNestedObject;
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::UndoType,
  protocol::verification::verify_urls_match,
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, person::ApubPerson},
  utils::protocol::InCommunity,
};
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoVote {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) object: IdOrNestedObject<Vote>,
  #[serde(rename = "type")]
  pub(crate) kind: UndoType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

impl InCommunity for UndoVote {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let object = self.object.dereference(context).await?;
    let community = object.community(context).await?;
    if let Some(audience) = &self.audience {
      verify_urls_match(audience.inner(), community.ap_id.inner())?;
    }
    Ok(community)
  }
}
