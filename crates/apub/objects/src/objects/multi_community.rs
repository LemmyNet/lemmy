use crate::{
  objects::ApubSite,
  protocol::multi_community::Feed,
  utils::functions::{GetActorType, check_apub_id_valid_with_strictness},
};
use activitypub_federation::{
  config::Data,
  protocol::verification::{verify_domains_match, verify_is_remote_object},
  traits::{Actor, Object},
};
use chrono::{DateTime, Utc};
use lemmy_api_utils::{context::LemmyContext, utils::slur_regex};
use lemmy_db_schema::{
  source::{
    multi_community::{MultiCommunity, MultiCommunityInsertForm},
    person::Person,
  },
  traits::ApubActor,
};
use lemmy_db_schema_file::enums::ActorType;
use lemmy_db_views_site::SiteView;
use lemmy_diesel_utils::{sensitive::SensitiveString, traits::Crud};
use lemmy_utils::{
  error::{LemmyError, LemmyErrorType, LemmyResult},
  utils::slurs::{check_slurs, check_slurs_opt},
};
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

  fn id(&self) -> &Url {
    self.ap_id.inner()
  }

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    Some(self.last_refreshed_at)
  }

  async fn read_from_id(
    object_id: Url,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<Option<Self>> {
    Ok(
      MultiCommunity::read_from_apub_id(&mut context.pool(), &object_id.into())
        .await?
        .map(Into::into),
    )
  }

  async fn delete(&self, _context: &Data<Self::DataType>) -> LemmyResult<()> {
    Err(LemmyErrorType::NotFound.into())
  }

  fn is_deleted(&self) -> bool {
    self.deleted
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
      following: self.following_url.clone().into(),
      name: self.name.clone(),
      summary: self.title.clone(),
      content: self.summary.clone(),
      attributed_to: creator.ap_id.into(),
    })
  }

  async fn verify(
    json: &Self::Kind,
    expected_domain: &Url,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    check_apub_id_valid_with_strictness(json.id.inner(), true, context).await?;
    verify_domains_match(expected_domain, json.id.inner())?;
    verify_is_remote_object(&json.id, context)?;

    let slur_regex = slur_regex(context).await?;

    check_slurs(&json.name, &slur_regex)?;
    check_slurs_opt(&json.summary, &slur_regex)?;
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
      summary: json.content,
      public_key: json.public_key.public_key_pem,
      private_key: None,
      inbox_url: Some(json.inbox.into()),
      following_url: Some(json.following.clone().into()),
      last_refreshed_at: Some(Utc::now()),
    };

    let multi = MultiCommunity::upsert(&mut context.pool(), &form)
      .await?
      .into();
    json.following.dereference(&multi, context).await?;
    Ok(multi)
  }
}

impl Actor for ApubMultiCommunity {
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

impl GetActorType for ApubMultiCommunity {
  fn actor_type(&self) -> ActorType {
    ActorType::MultiCommunity
  }
}
