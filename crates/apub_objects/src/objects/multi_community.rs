use crate::protocol::multi_community::Feed;
use activitypub_federation::{
  config::Data,
  protocol::verification::verify_domains_match,
  traits::Object,
};
use chrono::{DateTime, Utc};
use lemmy_api_common::{context::LemmyContext, LemmyErrorType};
use lemmy_db_schema::source::multi_community::{
  MultiCommunity,
  MultiCommunityApub,
  MultiCommunityInsertForm,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubMultiCommunity(pub MultiCommunity);

impl Deref for ApubMultiCommunity {
  type Target = MultiCommunity;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

/// TODO: This should use Collection instead of Object, but then it would not work with
/// resolve_object. Anyway the Collection trait is not working well and should be rewritten
/// in the library.
#[async_trait::async_trait]
impl Object for ApubMultiCommunity {
  type DataType = LemmyContext;
  type Kind = Feed;
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
    verify_domains_match(expected_domain, json.id.inner())?;
    Ok(())
  }

  async fn from_json(json: Self::Kind, context: &Data<LemmyContext>) -> LemmyResult<Self> {
    let creator = json.attributed_to.dereference(context).await?;
    let form = MultiCommunityInsertForm {
      creator_id: creator.id,
      instance_id: creator.instance_id,
      name: json.name,
      ap_id: json.id.into(),
      local: Some(false),
      title: json.summary,
      description: json.content,
    };

    let multi = ApubMultiCommunity(MultiCommunityApub::upsert(&mut context.pool(), &form).await?);
    json.following.dereference(&multi, context).await?;
    Ok(multi)
  }
}
