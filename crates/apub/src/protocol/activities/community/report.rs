use crate::{
  activities::verify_community_matches,
  fetcher::post_or_comment::PostOrComment,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::InCommunity,
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::FlagType,
  protocol::helpers::deserialize_one,
};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::{LemmyError, LemmyResult};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one")]
  pub(crate) to: [ObjectId<ApubCommunity>; 1],
  pub(crate) object: ReportObject,
  pub(crate) summary: String,
  #[serde(rename = "type")]
  pub(crate) kind: FlagType,
  pub(crate) id: Url,
  pub(crate) audience: Option<ObjectId<ApubCommunity>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum ReportObject {
  Lemmy(ObjectId<PostOrComment>),
  /// Mastodon sends an array containing post id and user id
  Mastodon([Url; 2]),
}

impl ReportObject {
  pub async fn dereference(self, context: &Data<LemmyContext>) -> LemmyResult<PostOrComment> {
    match self {
      ReportObject::Lemmy(l) => l.dereference(context).await,
      ReportObject::Mastodon(m) => ObjectId::from(m[0].clone()).dereference(context).await,
    }
  }
}

#[async_trait::async_trait]
impl InCommunity for Report {
  async fn community(&self, context: &Data<LemmyContext>) -> Result<ApubCommunity, LemmyError> {
    let community = self.to[0].dereference(context).await?;
    if let Some(audience) = &self.audience {
      verify_community_matches(audience, community.actor_id.clone())?;
    }
    Ok(community)
  }
}
