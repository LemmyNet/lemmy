use crate::{
  objects::community::ApubCommunity,
  protocol::collections::group_followers::GroupFollowers,
};
use activitypub_federation::{
  config::Data,
  kinds::collection::CollectionType,
  protocol::verification::verify_domains_match,
  traits::Collection,
};
use lemmy_api_common::{context::LemmyContext, utils::generate_followers_url};
use lemmy_db_schema::aggregates::structs::CommunityAggregates;
use lemmy_db_views_actor::structs::CommunityFollowerView;
use lemmy_utils::error::LemmyError;
use url::Url;

#[derive(Clone, Debug)]
pub(crate) struct ApubCommunityFollower(());

#[async_trait::async_trait]
impl Collection for ApubCommunityFollower {
  type Owner = ApubCommunity;
  type DataType = LemmyContext;
  type Kind = GroupFollowers;
  type Error = LemmyError;

  async fn read_local(
    community: &Self::Owner,
    context: &Data<Self::DataType>,
  ) -> Result<Self::Kind, Self::Error> {
    let community_id = community.id;
    let community_followers =
      CommunityFollowerView::count_community_followers(&mut context.pool(), community_id).await?;

    Ok(GroupFollowers {
      id: generate_followers_url(&community.actor_id)?.into(),
      r#type: CollectionType::Collection,
      total_items: community_followers as i32,
      items: vec![],
    })
  }

  async fn verify(
    json: &Self::Kind,
    expected_domain: &Url,
    _data: &Data<Self::DataType>,
  ) -> Result<(), Self::Error> {
    verify_domains_match(expected_domain, &json.id)?;
    Ok(())
  }

  async fn from_json(
    json: Self::Kind,
    community: &Self::Owner,
    context: &Data<Self::DataType>,
  ) -> Result<Self, Self::Error> {
    CommunityAggregates::update_federated_followers(
      &mut context.pool(),
      community.id,
      json.total_items,
    )
    .await?;

    Ok(ApubCommunityFollower(()))
  }
}
