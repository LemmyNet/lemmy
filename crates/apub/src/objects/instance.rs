use super::verify_is_remote_object;
use crate::{
  activities::GetActorType,
  check_apub_id_valid_with_strictness,
  local_site_data_cached,
  objects::read_from_string_or_source_opt,
  protocol::{
    objects::{instance::Instance, LanguageTag},
    ImageObject,
    Source,
  },
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::actor::ApplicationType,
  protocol::{values::MediaTypeHtml, verification::verify_domains_match},
  traits::{Actor, Object},
};
use chrono::{DateTime, Utc};
use lemmy_api_common::{
  context::LemmyContext,
  utils::{
    get_url_blocklist,
    local_site_opt_to_slur_regex,
    process_markdown_opt,
    proxy_image_link_opt_apub,
  },
};
use lemmy_db_schema::{
  newtypes::InstanceId,
  sensitive::SensitiveString,
  source::{
    activity::ActorType,
    actor_language::SiteLanguage,
    instance::Instance as DbInstance,
    local_site::LocalSite,
    site::{Site, SiteInsertForm},
  },
  traits::Crud,
  utils::naive_now,
};
use lemmy_utils::{
  error::{LemmyError, LemmyResult},
  utils::{
    markdown::markdown_to_html,
    slurs::{check_slurs, check_slurs_opt},
  },
  LemmyErrorType,
};
use std::ops::Deref;
use tracing::debug;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubSite(Site);

impl Deref for ApubSite {
  type Target = Site;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<Site> for ApubSite {
  fn from(s: Site) -> Self {
    ApubSite(s)
  }
}

#[async_trait::async_trait]
impl Object for ApubSite {
  type DataType = LemmyContext;
  type Kind = Instance;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    Some(self.last_refreshed_at)
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_id(object_id: Url, data: &Data<Self::DataType>) -> LemmyResult<Option<Self>> {
    Ok(
      Site::read_from_apub_id(&mut data.pool(), &object_id.into())
        .await?
        .map(Into::into),
    )
  }

  async fn delete(self, _data: &Data<Self::DataType>) -> LemmyResult<()> {
    unimplemented!()
  }

  #[tracing::instrument(skip_all)]
  async fn into_json(self, data: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    let site_id = self.id;
    let langs = SiteLanguage::read(&mut data.pool(), site_id).await?;
    let language = LanguageTag::new_multiple(langs, &mut data.pool()).await?;

    let instance = Instance {
      kind: ApplicationType::Application,
      id: self.id().into(),
      name: self.name.clone(),
      preferred_username: Some(data.domain().to_string()),
      content: self.sidebar.as_ref().map(|d| markdown_to_html(d)),
      source: self.sidebar.clone().map(Source::new),
      summary: self.description.clone(),
      media_type: self.sidebar.as_ref().map(|_| MediaTypeHtml::Html),
      icon: self.icon.clone().map(ImageObject::new),
      image: self.banner.clone().map(ImageObject::new),
      inbox: self.inbox_url.clone().into(),
      outbox: Url::parse(&format!("{}/site_outbox", self.actor_id))?,
      public_key: self.public_key(),
      language,
      content_warning: self.content_warning.clone(),
      published: self.published,
      updated: self.updated,
    };
    Ok(instance)
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    apub: &Self::Kind,
    expected_domain: &Url,
    data: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    check_apub_id_valid_with_strictness(apub.id.inner(), true, data).await?;
    verify_domains_match(expected_domain, apub.id.inner())?;
    verify_is_remote_object(&apub.id, data)?;

    let local_site_data = local_site_data_cached(&mut data.pool()).await?;
    let slur_regex = &local_site_opt_to_slur_regex(&local_site_data.local_site);
    check_slurs(&apub.name, slur_regex)?;
    check_slurs_opt(&apub.summary, slur_regex)?;

    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn from_json(apub: Self::Kind, context: &Data<Self::DataType>) -> LemmyResult<Self> {
    let domain = apub
      .id
      .inner()
      .domain()
      .ok_or(LemmyErrorType::UrlWithoutDomain)?;
    let instance = DbInstance::read_or_create(&mut context.pool(), domain.to_string()).await?;

    let local_site = LocalSite::read(&mut context.pool()).await.ok();
    let slur_regex = &local_site_opt_to_slur_regex(&local_site);
    let url_blocklist = get_url_blocklist(context).await?;
    let sidebar = read_from_string_or_source_opt(&apub.content, &None, &apub.source);
    let sidebar = process_markdown_opt(&sidebar, slur_regex, &url_blocklist, context).await?;
    let icon = proxy_image_link_opt_apub(apub.icon.map(|i| i.url), context).await?;
    let banner = proxy_image_link_opt_apub(apub.image.map(|i| i.url), context).await?;

    let site_form = SiteInsertForm {
      name: apub.name.clone(),
      sidebar,
      updated: apub.updated,
      icon,
      banner,
      description: apub.summary,
      actor_id: Some(apub.id.clone().into()),
      last_refreshed_at: Some(naive_now()),
      inbox_url: Some(apub.inbox.clone().into()),
      public_key: Some(apub.public_key.public_key_pem.clone()),
      private_key: None,
      instance_id: instance.id,
      content_warning: apub.content_warning,
    };
    let languages =
      LanguageTag::to_language_id_multiple(apub.language, &mut context.pool()).await?;

    let site = Site::create(&mut context.pool(), &site_form).await?;
    SiteLanguage::update(&mut context.pool(), languages, &site).await?;
    Ok(site.into())
  }
}

impl Actor for ApubSite {
  fn id(&self) -> Url {
    self.actor_id.inner().clone()
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
}
impl GetActorType for ApubSite {
  fn actor_type(&self) -> ActorType {
    ActorType::Site
  }
}

/// Try to fetch the instance actor (to make things like instance rules available).
pub(in crate::objects) async fn fetch_instance_actor_for_object<T: Into<Url> + Clone>(
  object_id: &T,
  context: &Data<LemmyContext>,
) -> LemmyResult<InstanceId> {
  let object_id: Url = object_id.clone().into();
  let instance_id = Site::instance_actor_id_from_url(object_id);
  let site = ObjectId::<ApubSite>::from(instance_id.clone())
    .dereference(context)
    .await;
  match site {
    Ok(s) => Ok(s.instance_id),
    Err(e) => {
      // Failed to fetch instance actor, its probably not a lemmy instance
      debug!("Failed to dereference site for {}: {}", &instance_id, e);
      let domain = instance_id
        .domain()
        .ok_or(LemmyErrorType::UrlWithoutDomain)?;
      Ok(
        DbInstance::read_or_create(&mut context.pool(), domain.to_string())
          .await?
          .id,
      )
    }
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::protocol::tests::file_to_json_object;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  pub(crate) async fn parse_lemmy_instance(context: &Data<LemmyContext>) -> LemmyResult<ApubSite> {
    let json: Instance = file_to_json_object("assets/lemmy/objects/instance.json")?;
    let id = Url::parse("https://enterprise.lemmy.ml/")?;
    ApubSite::verify(&json, &id, context).await?;
    let site = ApubSite::from_json(json, context).await?;
    assert_eq!(context.request_count(), 0);
    Ok(site)
  }

  #[tokio::test]
  #[serial]
  async fn test_parse_lemmy_instance() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let site = parse_lemmy_instance(&context).await?;

    assert_eq!(site.name, "Enterprise");
    assert_eq!(
      site.description.as_ref().map(std::string::String::len),
      Some(15)
    );

    Site::delete(&mut context.pool(), site.id).await?;
    Ok(())
  }
}
