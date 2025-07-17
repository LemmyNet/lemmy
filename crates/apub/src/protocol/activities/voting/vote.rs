use activitypub_federation::{config::Data, fetch::object_id::ObjectId};
use either::Either;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, person::ApubPerson, PostOrComment},
  utils::protocol::InCommunity,
};
use lemmy_db_schema::{source::community::Community, traits::Crud};
use lemmy_db_views_post::PostView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::{FederationError, LemmyError, LemmyResult};
use serde::{Deserialize, Serialize};
use strum::Display;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vote {
  pub(crate) actor: ObjectId<ApubPerson>,
  pub(crate) object: ObjectId<PostOrComment>,
  #[serde(rename = "type")]
  pub(crate) kind: VoteType,
  pub(crate) id: Url,
}

#[derive(Clone, Debug, Display, Deserialize, Serialize, PartialEq, Eq)]
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
      _ => Err(FederationError::InvalidVoteValue.into()),
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

impl InCommunity for Vote {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let community = match self.object.dereference(context).await? {
      Either::Left(p) => Community::read(&mut context.pool(), p.community_id).await?,
      Either::Right(c) => {
        let site_view = SiteView::read_local(&mut context.pool()).await?;
        PostView::read(
          &mut context.pool(),
          c.post_id,
          None,
          site_view.instance.id,
          false,
        )
        .await?
        .community
      }
    };
    Ok(community.into())
  }
}
