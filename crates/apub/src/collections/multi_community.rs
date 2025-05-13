use crate::protocol::collections::multi_community::MultiCommunityCollection;
use activitypub_federation::{
  config::Data,
  kinds::collection::CollectionType,
  protocol::verification::verify_domains_match,
  traits::Collection,
};
use futures::future::join_all;
use lemmy_api_common::context::LemmyContext;
use lemmy_apub_objects::objects::person::ApubPerson;
use lemmy_db_schema::{
  newtypes::{CommunityId, MultiCommunityId},
  source::multi_community::MultiCommunity,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

#[derive(Clone, Debug)]
pub(crate) struct ApubMultiCommunity(());

// TODO
const MULTI_ID: MultiCommunityId = MultiCommunityId(0);

/// TODO: This trait is awkward to use and needs to be rewritten in the library
#[async_trait::async_trait]
impl Collection for ApubMultiCommunity {
  type Owner = (ApubPerson, MultiCommunityId);
  type DataType = LemmyContext;
  type Kind = MultiCommunityCollection;
  type Error = LemmyError;

  async fn read_local(
    owner: &Self::Owner,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<Self::Kind> {
    let multi = MultiCommunity::read(&mut context.pool(), owner.1).await?;
    Ok(MultiCommunityCollection {
      r#type: CollectionType::Collection,
      id: Url::parse(&format!("{}/m/{}", owner.0.ap_id, multi.multi.name))?,
      total_items: multi.entries.len() as i32,
      items: multi.entries,
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
    .flat_map(|c: LemmyResult<CommunityId>| c.ok())
    .collect();

    MultiCommunity::update(&mut context.pool(), owner.1, communities).await?;

    // This return value is unused, so just set an empty vec
    Ok(ApubMultiCommunity(()))
  }
}
