use crate::{
  protocol::instance::Instance,
  utils::{
    functions::{
      GetActorType,
      check_apub_id_valid_with_strictness,
      read_from_string_or_source_opt,
    },
    markdown_links::markdown_rewrite_remote_links_opt,
    protocol::{ImageObject, LanguageTag, Source},
  },
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::actor::ApplicationType,
  protocol::{
    values::MediaTypeHtml,
    verification::{verify_domains_match, verify_is_remote_object},
  },
  traits::{Actor, Object},
};
use chrono::{DateTime, Utc};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{get_url_blocklist, process_markdown_opt, proxy_image_link_opt_apub, slur_regex},
};
use lemmy_db_schema::source::{
  actor_language::SiteLanguage,
  instance::Instance as DbInstance,
  site::{Site, SiteInsertForm},
};
use lemmy_db_schema_file::{InstanceId, enums::ActorType};
use lemmy_diesel_utils::{sensitive::SensitiveString, traits::Crud};
use lemmy_utils::{
  error::{LemmyError, LemmyResult, UntranslatedError},
  utils::{
    markdown::markdown_to_html,
    slurs::{check_slurs, check_slurs_opt},
  },
};
use std::ops::Deref;
use tracing::debug;
use url::Url;

#[derive(Clone, Debug)]
pub struct ApubSite(pub Site);

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

  fn id(&self) -> &Url {
    self.ap_id.inner()
  }

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    Some(self.last_refreshed_at)
  }

  async fn read_from_id(object_id: Url, data: &Data<Self::DataType>) -> LemmyResult<Option<Self>> {
    Ok(
      Site::read_from_apub_id(&mut data.pool(), &object_id.into())
        .await?
        .map(Into::into),
    )
  }

  async fn delete(&self, _data: &Data<Self::DataType>) -> LemmyResult<()> {
    Err(UntranslatedError::CantDeleteSite.into())
  }

  async fn into_json(self, data: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    let site_id = self.id;
    let langs = SiteLanguage::read(&mut data.pool(), site_id).await?;
    let language = LanguageTag::new_multiple(langs, &mut data.pool()).await?;

    let instance = Instance {
      kind: ApplicationType::Application,
      id: self.id().clone().into(),
      name: self.name.clone(),
      preferred_username: Some(data.domain().to_string()),
      summary: self.sidebar.as_ref().map(|d| markdown_to_html(d)),
      source: self.sidebar.clone().map(Source::new),
      content: self.summary.clone(),
      media_type: self.sidebar.as_ref().map(|_| MediaTypeHtml::Html),
      icon: self.icon.clone().map(ImageObject::new),
      image: self.banner.clone().map(ImageObject::new),
      inbox: self.inbox_url.clone().into(),
      outbox: Url::parse(&format!("{}site_outbox", self.ap_id))?,
      public_key: self.public_key(),
      language,
      content_warning: self.content_warning.clone(),
      published: self.published_at,
      updated: self.updated_at,
    };
    Ok(instance)
  }

  async fn verify(
    apub: &Self::Kind,
    expected_domain: &Url,
    data: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    check_apub_id_valid_with_strictness(apub.id.inner(), true, data).await?;
    verify_domains_match(expected_domain, apub.id.inner())?;
    verify_is_remote_object(&apub.id, data)?;

    let slur_regex = &slur_regex(data).await?;
    check_slurs(&apub.name, slur_regex)?;
    check_slurs_opt(&apub.summary, slur_regex)?;

    Ok(())
  }

  async fn from_json(apub: Self::Kind, context: &Data<Self::DataType>) -> LemmyResult<Self> {
    let domain = apub
      .id
      .inner()
      .domain()
      .ok_or(UntranslatedError::UrlWithoutDomain)?;
    let instance = DbInstance::read_or_create(&mut context.pool(), domain).await?;

    let slur_regex = slur_regex(context).await?;
    let url_blocklist = get_url_blocklist(context).await?;
    let sidebar = read_from_string_or_source_opt(&apub.content, &None, &apub.source);
    let sidebar = process_markdown_opt(&sidebar, &slur_regex, &url_blocklist, context).await?;
    let sidebar = markdown_rewrite_remote_links_opt(sidebar, context).await;
    let icon = proxy_image_link_opt_apub(apub.icon.map(|i| i.url), context).await?;
    let banner = proxy_image_link_opt_apub(apub.image.map(|i| i.url), context).await?;

    let site_form = SiteInsertForm {
      name: apub.name.clone(),
      sidebar,
      updated_at: apub.updated,
      icon,
      banner,
      summary: apub.summary,
      ap_id: Some(apub.id.clone().into()),
      last_refreshed_at: Some(Utc::now()),
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
pub(crate) async fn fetch_instance_actor_for_object<T: Into<Url> + Clone>(
  object_id: &T,
  context: &Data<LemmyContext>,
) -> LemmyResult<InstanceId> {
  let object_id: Url = object_id.clone().into();
  let instance_id = Site::instance_ap_id_from_url(object_id);
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
        .ok_or(UntranslatedError::UrlWithoutDomain)?;
      Ok(
        DbInstance::read_or_create(&mut context.pool(), domain)
          .await?
          .id,
      )
    }
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::utils::test::parse_lemmy_instance;
  use lemmy_db_schema::source::instance::Instance;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_parse_lemmy_instance() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let site = parse_lemmy_instance(&context).await?;

    assert_eq!(site.name, "Enterprise");
    assert_eq!(
      site.summary.as_ref().map(std::string::String::len),
      Some(15)
    );

    Instance::delete_all(&mut context.pool()).await?;
    Ok(())
  }
}
