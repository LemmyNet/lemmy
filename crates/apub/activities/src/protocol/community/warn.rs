use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  protocol::{helpers::deserialize_one, verification::verify_urls_match},
};
use either::Either;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{PostOrComment, community::ApubCommunity, person::ApubPerson},
  utils::protocol::InCommunity,
};
use lemmy_db_schema::source::{community::Community, post::Post};
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use strum::Display;
use url::Url;

#[derive(Clone, Debug, Display, Deserialize, Serialize)]
pub enum WarnType {
  Warn,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Warn {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one")]
  pub(crate) to: [ObjectId<ApubPerson>; 1],
  pub(crate) object: ObjectId<PostOrComment>,
  pub(crate) summary: String,
  #[serde(rename = "type")]
  pub(crate) kind: WarnType,
  pub(crate) id: Url,
  pub(crate) audience: ObjectId<ApubCommunity>,
}

impl InCommunity for Warn {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let community_id = match self.object.dereference(context).await? {
      Either::Left(post) => post.community_id,
      Either::Right(comment) => {
        let post = Post::read(&mut context.pool(), comment.post_id).await?;
        post.community_id
      }
    };
    let community = Community::read(&mut context.pool(), community_id).await?;
    verify_urls_match(self.audience.inner(), community.ap_id.inner())?;

    Ok(community.into())
  }
}
