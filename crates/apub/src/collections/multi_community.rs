use crate::protocol::collections::multi_community::MultiCommunityCollection;
use activitypub_federation::{
  config::Data,
  kinds::collection::CollectionType,
  protocol::verification::verify_domains_match,
  traits::{Collection, Object},
};
use chrono::{DateTime, Utc};
use futures::future::join_all;
use lemmy_api_common::{context::LemmyContext, LemmyErrorType};
use lemmy_db_schema::{
  impls::multi_community::ReadParams,
  newtypes::{CommunityId, MultiCommunityId},
  source::multi_community::MultiCommunity,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use tracing::info;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubMultiCommunity(pub MultiCommunityId);

/// TODO: This trait is awkward to use and needs to be rewritten in the library
#[async_trait::async_trait]
impl Collection for ApubMultiCommunity {
  type Owner = MultiCommunityId;
  type DataType = LemmyContext;
  type Kind = MultiCommunityCollection;
  type Error = LemmyError;

  async fn read_local(
    owner: &Self::Owner,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<Self::Kind> {
    let multi = MultiCommunity::read_apub(&mut context.pool(), *owner).await?;
    Ok(MultiCommunityCollection {
      r#type: CollectionType::Collection,
      id: multi.multi.ap_id.into(),
      total_items: multi.entries.len().try_into()?,
      items: multi.entries.into_iter().map(Into::into).collect(),
    })
  }

  async fn verify(
    collection: &MultiCommunityCollection,
    expected_domain: &Url,
    _data: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    verify_domains_match(expected_domain, &collection.id)?;
    Ok(())
  }

  async fn from_json(
    apub: Self::Kind,
    owner: &Self::Owner,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<Self> {
    let communities = join_all(
      apub
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

    MultiCommunity::update(&mut context.pool(), *owner, communities).await?;

    // This return value is unused, so just set an empty vec
    Ok(ApubMultiCommunity(*owner))
  }
}

/// Workaround so that this can be fetched in a single http request together with post,
/// comment etc for resolve_object
#[async_trait::async_trait]
impl Object for ApubMultiCommunity {
  type DataType = LemmyContext;
  type Kind = MultiCommunityCollection;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    None
  }

  async fn read_from_id(
    _object_id: Url,
    _context: &Data<Self::DataType>,
  ) -> LemmyResult<Option<Self>> {
    Err(LemmyErrorType::NotFound.into())
  }

  async fn delete(self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    Err(LemmyErrorType::NotFound.into())
  }

  async fn into_json(self, _context: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    Err(LemmyErrorType::NotFound.into())
  }
  async fn verify(
    json: &Self::Kind,
    expected_domain: &Url,
    _context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    verify_domains_match(expected_domain, &json.id)?;
    Ok(())
  }

  async fn from_json(json: Self::Kind, context: &Data<LemmyContext>) -> LemmyResult<Self> {
    let multi = MultiCommunity::read(
      &mut context.pool(),
      ReadParams::ApId(json.id.clone().into()),
    )
    .await?;
    <ApubMultiCommunity as Collection>::from_json(json, &multi.multi.id, context).await?;
    Ok(ApubMultiCommunity(multi.multi.id))
  }
}
