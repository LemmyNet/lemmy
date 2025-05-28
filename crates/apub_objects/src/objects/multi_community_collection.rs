use super::multi_community::ApubMultiCommunity;
use crate::protocol::multi_community::FeedCollection;
use activitypub_federation::{
  config::Data,
  protocol::verification::verify_domains_match,
  traits::Collection,
};
use futures::future::join_all;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{newtypes::CommunityId, source::multi_community::MultiCommunityApub};
use lemmy_utils::error::{LemmyError, LemmyResult};
use tracing::info;
use url::Url;

pub struct ApubFeedCollection;

#[async_trait::async_trait]
impl Collection for ApubFeedCollection {
  type DataType = LemmyContext;
  type Kind = FeedCollection;
  type Owner = ApubMultiCommunity;
  type Error = LemmyError;

  async fn read_local(
    owner: &Self::Owner,
    context: &Data<Self::DataType>,
  ) -> Result<Self::Kind, Self::Error> {
    // TODO: this only needs to return entries
    let multi = MultiCommunityApub::read_local(&mut context.pool(), &owner.name).await?;
    Ok(Self::Kind {
      r#type: Default::default(),
      id: owner.following_url()?,
      total_items: multi.entries.len().try_into()?,
      items: multi.entries.into_iter().map(Into::into).collect(),
    })
  }

  async fn verify(
    json: &Self::Kind,
    expected_domain: &Url,
    _context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    verify_domains_match(expected_domain, &json.id.clone().into())?;
    Ok(())
  }

  async fn from_json(
    json: Self::Kind,
    owner: &Self::Owner,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<Self> {
    let communities = join_all(
      json
        .items
        .into_iter()
        .map(|ap_id| async move { Ok(ap_id.dereference(context).await?.id) }),
    )
    .await
    .into_iter()
    .flat_map(|c: LemmyResult<CommunityId>| match c {
      Ok(c) => Some(c),
      Err(e) => {
        info!("Failed to fetch multi-community item: {e}");
        None
      }
    })
    .collect();

    MultiCommunityApub::update_entries(&mut context.pool(), owner.id, &communities).await?;

    // TODO: local users who followed the multi-comm need to have community follows updated here

    Ok(ApubFeedCollection)
  }
}
