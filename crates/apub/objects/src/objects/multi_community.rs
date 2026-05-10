use crate::{
  objects::ApubSite,
  protocol::multi_community::Feed,
  utils::{
    functions::{
      GetActorType,
      check_apub_id_valid_with_strictness,
      read_from_string_or_source_opt,
    },
    markdown_links::markdown_rewrite_remote_links_opt,
    protocol::Source,
  },
};
use activitypub_federation::{
  config::Data,
  protocol::{
    values::MediaTypeHtml,
    verification::{verify_domains_match, verify_is_remote_object},
  },
  traits::{Actor, Object},
};
use chrono::{DateTime, Utc};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{process_markdown_opt, slur_regex},
};
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
  utils::{
    markdown::markdown_to_html,
    slurs::remove_slurs,
    validation::{
      is_valid_body_field,
      is_valid_display_name,
      summary_length_check,
      truncate_summary,
    },
  },
};
use regex::RegexSet;
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
      preferred_username: self.name.clone(),
      name: self.title.clone(),
      summary: self.sidebar.as_ref().map(|d| markdown_to_html(d)),
      source: self.sidebar.clone().map(Source::new),
      description: self.summary.clone(),
      media_type: self.sidebar.as_ref().map(|_| MediaTypeHtml::Html),
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

    Ok(())
  }

  async fn from_json(json: Self::Kind, context: &Data<LemmyContext>) -> LemmyResult<Self> {
    let creator = json.attributed_to.dereference(context).await?;
    let slur_regex = slur_regex(context).await?;
    let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;

    // Use empty regex so that url blocklist doesnt prevent community federation.
    let url_blocklist = RegexSet::empty();

    let sidebar = read_from_string_or_source_opt(&json.summary, &None, &json.source);
    let sidebar =
      process_markdown_opt(&sidebar, &slur_regex, &url_blocklist, &local_site, context).await?;
    let sidebar = markdown_rewrite_remote_links_opt(sidebar, context).await;
    if let Some(sidebar) = &sidebar {
      is_valid_body_field(sidebar, false)?;
    }

    let summary = json
      .description
      .clone()
      .as_deref()
      .map(truncate_summary)
      .map(|s| remove_slurs(&s, &slur_regex));
    if let Some(summary) = &summary {
      summary_length_check(summary)?;
    }

    let name = json.preferred_username.clone();
    let title = json.name.map(|t| remove_slurs(&t, &slur_regex));
    if let Some(title) = &title {
      is_valid_display_name(title)?;
    }

    let form = MultiCommunityInsertForm {
      creator_id: creator.id,
      instance_id: creator.instance_id,
      name,
      ap_id: Some(json.id.into()),
      local: Some(false),
      title,
      summary,
      sidebar,
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
