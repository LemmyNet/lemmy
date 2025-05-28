use super::multi_community_collection::ApubFeedCollection;
use crate::{objects::ApubSite, protocol::multi_community::Feed};
use activitypub_federation::{
  config::Data,
  fetch::collection_id::CollectionId,
  protocol::verification::verify_domains_match,
  traits::{Actor, Object},
};
use chrono::{DateTime, Utc};
use lemmy_api_common::{context::LemmyContext, LemmyErrorType};
use lemmy_db_schema::{
  sensitive::SensitiveString,
  source::{
    multi_community::{MultiCommunity, MultiCommunityApub, MultiCommunityInsertForm},
    person::Person,
  },
  traits::Crud,
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::{LemmyError, LemmyResult};
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubMultiCommunity(MultiCommunity);

impl Deref for ApubMultiCommunity {
  type Target = MultiCommunity;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl ApubMultiCommunity {
  pub fn following_url(&self) -> LemmyResult<CollectionId<ApubFeedCollection>> {
    Ok(Url::parse(&format!("{}/following", self.ap_id))?.into())
  }
}

impl From<MultiCommunity> for ApubMultiCommunity {
  fn from(m: MultiCommunity) -> Self {
    ApubMultiCommunity(m)
  }
}

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

  async fn into_json(self, context: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    let site_view = SiteView::read_local(&mut context.pool()).await?;
    let site = ApubSite(site_view.site.clone());
    let creator = Person::read(&mut context.pool(), self.creator_id).await?;
    Ok(Feed {
      r#type: Default::default(),
      id: self.ap_id.clone().into(),
      inbox: site_view.site.inbox_url.into(),
      // reusing pubkey from site instead of generating new one
      public_key: site.public_key(),
      following: self.following_url()?,
      name: self.name.clone(),
      summary: self.title.clone(),
      content: self.description.clone(),
      attributed_to: creator.ap_id.into(),
    })
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
      ap_id: Some(json.id.into()),
      local: Some(false),
      title: json.summary,
      description: json.content,
      public_key: Some(json.public_key.public_key_pem),
      private_key: None,
      inbox_url: Some(json.inbox.into()),
    };

    let multi = MultiCommunityApub::upsert(&mut context.pool(), &form)
      .await?
      .into();
    json.following.dereference(&multi, context).await?;
    Ok(multi)
  }
}

impl Actor for ApubMultiCommunity {
  fn id(&self) -> Url {
    self.ap_id.inner().clone()
  }

  fn public_key_pem(&self) -> &str {
    &self.public_key
  }

  fn private_key_pem(&self) -> Option<String> {
    self.private_key.clone().map(SensitiveString::into_inner)
  }

  fn inbox(&self) -> Url {
    self.inbox_url.clone().into()
  }

  fn shared_inbox(&self) -> Option<Url> {
    None
  }
}
