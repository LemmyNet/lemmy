use crate::protocol::collections::multi_community::MultiCommunityCollection;
use activitypub_federation::{
  config::Data,
  protocol::verification::verify_domains_match,
  traits::Object,
};
use chrono::{DateTime, Utc};
use futures::future::join_all;
use lemmy_api_common::{context::LemmyContext, LemmyErrorType};
use lemmy_db_schema::{
  newtypes::{CommunityId, MultiCommunityId},
  source::multi_community::{MultiCommunity, MultiCommunityInsertForm},
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use tracing::info;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubMultiCommunity(pub MultiCommunityId);

/// TODO: This should use Collection instead of Object, but then it would not work with
/// resolve_object. Anyway the Collection trait is not working well and should be rewritten
/// in the library.
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
    Ok(None)
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
    let owner = json.attributed_to.dereference(context).await?;
    let form = MultiCommunityInsertForm {
      owner_id: owner.id,
      name: json.name,
      ap_id: json.id.into(),
    };
    let multi = MultiCommunity::upsert(&mut context.pool(), &form).await?;

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

    MultiCommunity::update(&mut context.pool(), multi.id, &communities).await?;

    Ok(ApubMultiCommunity(multi.id))
  }
}
