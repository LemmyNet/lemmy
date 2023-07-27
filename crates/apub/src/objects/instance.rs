use crate::{
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
use chrono::NaiveDateTime;
use lemmy_api_common::{
  context::LemmyContext,
  utils::{local_site_opt_to_slur_regex, sanitize_html_opt},
};
use lemmy_db_schema::{
  newtypes::InstanceId,
  source::{
    actor_language::SiteLanguage,
    instance::Instance as DbInstance,
    site::{Site, SiteInsertForm},
  },
  traits::Crud,
  utils::{naive_now, DbPool},
};
use lemmy_utils::{
  error::LemmyError,
  utils::{
    markdown::markdown_to_html,
    slurs::{check_slurs, check_slurs_opt},
    time::convert_datetime,
  },
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

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    Some(self.last_refreshed_at)
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_id(
    object_id: Url,
    data: &Data<Self::DataType>,
  ) -> Result<Option<Self>, LemmyError> {
    Ok(
      Site::read_from_apub_id(&mut data.pool(), &object_id.into())
        .await?
        .map(Into::into),
    )
  }

  async fn delete(self, _data: &Data<Self::DataType>) -> Result<(), LemmyError> {
    unimplemented!()
  }

  #[tracing::instrument(skip_all)]
  async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, LemmyError> {
    let site_id = self.id;
    let langs = SiteLanguage::read(&mut data.pool(), site_id).await?;
    let language = LanguageTag::new_multiple(langs, &mut data.pool()).await?;

    let instance = Instance {
      kind: ApplicationType::Application,
      id: self.id().into(),
      name: self.name.clone(),
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
      published: convert_datetime(self.published),
      updated: self.updated.map(convert_datetime),
    };
    Ok(instance)
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    apub: &Self::Kind,
    expected_domain: &Url,
    data: &Data<Self::DataType>,
  ) -> Result<(), LemmyError> {
    check_apub_id_valid_with_strictness(apub.id.inner(), true, data).await?;
    verify_domains_match(expected_domain, apub.id.inner())?;

    let local_site_data = local_site_data_cached(&mut data.pool()).await?;
    let slur_regex = &local_site_opt_to_slur_regex(&local_site_data.local_site);
    check_slurs(&apub.name, slur_regex)?;
    check_slurs_opt(&apub.summary, slur_regex)?;

    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn from_json(apub: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, LemmyError> {
    let domain = apub.id.inner().domain().expect("group id has domain");
    let instance = DbInstance::read_or_create(&mut data.pool(), domain.to_string()).await?;

    let sidebar = read_from_string_or_source_opt(&apub.content, &None, &apub.source);
    let sidebar = sanitize_html_opt(&sidebar);
    let description = sanitize_html_opt(&apub.summary);

    let site_form = SiteInsertForm {
      name: apub.name.clone(),
      sidebar,
      updated: apub.updated.map(|u| u.clone().naive_local()),
      icon: apub.icon.clone().map(|i| i.url.into()),
      banner: apub.image.clone().map(|i| i.url.into()),
      description,
      actor_id: Some(apub.id.clone().into()),
      last_refreshed_at: Some(naive_now()),
      inbox_url: Some(apub.inbox.clone().into()),
      public_key: Some(apub.public_key.public_key_pem.clone()),
      private_key: None,
      instance_id: instance.id,
    };
    let languages = LanguageTag::to_language_id_multiple(apub.language, &mut data.pool()).await?;

    let site = Site::create(&mut data.pool(), &site_form).await?;
    SiteLanguage::update(&mut data.pool(), languages, &site).await?;
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
    self.private_key.clone()
  }

  fn inbox(&self) -> Url {
    self.inbox_url.clone().into()
  }
}

/// Try to fetch the instance actor (to make things like instance rules available).
pub(in crate::objects) async fn fetch_instance_actor_for_object<T: Into<Url> + Clone>(
  object_id: &T,
  context: &Data<LemmyContext>,
) -> Result<InstanceId, LemmyError> {
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
      let domain = instance_id.domain().expect("has domain");
      Ok(
        DbInstance::read_or_create(&mut context.pool(), domain.to_string())
          .await?
          .id,
      )
    }
  }
}

pub(crate) async fn remote_instance_inboxes(pool: &mut DbPool<'_>) -> Result<Vec<Url>, LemmyError> {
  Ok(
    Site::read_remote_sites(pool)
      .await?
      .into_iter()
      .map(|s| ApubSite::from(s).shared_inbox_or_inbox())
      .collect(),
  )
}

#[cfg(test)]
pub(crate) mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use super::*;
  use crate::{objects::tests::init_context, protocol::tests::file_to_json_object};
  use lemmy_db_schema::traits::Crud;
  use serial_test::serial;

  pub(crate) async fn parse_lemmy_instance(context: &Data<LemmyContext>) -> ApubSite {
    let json: Instance = file_to_json_object("assets/lemmy/objects/instance.json").unwrap();
    let id = Url::parse("https://enterprise.lemmy.ml/").unwrap();
    ApubSite::verify(&json, &id, context).await.unwrap();
    let site = ApubSite::from_json(json, context).await.unwrap();
    assert_eq!(context.request_count(), 0);
    site
  }

  #[tokio::test]
  #[serial]
  async fn test_parse_lemmy_instance() {
    let context = init_context().await;
    let site = parse_lemmy_instance(&context).await;

    assert_eq!(site.name, "Enterprise");
    assert_eq!(site.description.as_ref().unwrap().len(), 15);

    Site::delete(&mut context.pool(), site.id).await.unwrap();
  }
}
