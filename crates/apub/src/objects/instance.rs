use crate::{
  check_apub_id_valid_with_strictness,
  fetch_local_site_data,
  local_instance,
  objects::read_from_string_or_source_opt,
  protocol::{
    objects::{instance::Instance, LanguageTag},
    ImageObject,
    Source,
  },
  ActorType,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  deser::values::MediaTypeHtml,
  traits::{Actor, ApubObject},
  utils::verify_domains_match,
};
use activitystreams_kinds::actor::ApplicationType;
use chrono::NaiveDateTime;
use lemmy_api_common::{context::LemmyContext, utils::local_site_opt_to_slur_regex};
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
  utils::{check_slurs, check_slurs_opt, convert_datetime, markdown_to_html},
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

#[async_trait::async_trait(?Send)]
impl ApubObject for ApubSite {
  type DataType = LemmyContext;
  type ApubType = Instance;
  type DbType = Site;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    Some(self.last_refreshed_at)
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_apub_id(
    object_id: Url,
    data: &Self::DataType,
  ) -> Result<Option<Self>, LemmyError> {
    Ok(
      Site::read_from_apub_id(data.pool(), object_id)
        .await?
        .map(Into::into),
    )
  }

  async fn delete(self, _data: &Self::DataType) -> Result<(), LemmyError> {
    unimplemented!()
  }

  #[tracing::instrument(skip_all)]
  async fn into_apub(self, data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    let site_id = self.id;
    let langs = SiteLanguage::read(data.pool(), site_id).await?;
    let language = LanguageTag::new_multiple(langs, data.pool()).await?;

    let instance = Instance {
      kind: ApplicationType::Application,
      id: ObjectId::new(self.actor_id()),
      name: self.name.clone(),
      content: self.sidebar.as_ref().map(|d| markdown_to_html(d)),
      source: self.sidebar.clone().map(Source::new),
      summary: self.description.clone(),
      media_type: self.sidebar.as_ref().map(|_| MediaTypeHtml::Html),
      icon: self.icon.clone().map(ImageObject::new),
      image: self.banner.clone().map(ImageObject::new),
      inbox: self.inbox_url.clone().into(),
      outbox: Url::parse(&format!("{}/site_outbox", self.actor_id))?,
      public_key: self.get_public_key(),
      language,
      published: convert_datetime(self.published),
      updated: self.updated.map(convert_datetime),
    };
    Ok(instance)
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    apub: &Self::ApubType,
    expected_domain: &Url,
    data: &Self::DataType,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let local_site_data = fetch_local_site_data(data.pool()).await?;

    check_apub_id_valid_with_strictness(apub.id.inner(), true, &local_site_data, data.settings())?;
    verify_domains_match(expected_domain, apub.id.inner())?;

    let slur_regex = &local_site_opt_to_slur_regex(&local_site_data.local_site);

    check_slurs(&apub.name, slur_regex)?;
    check_slurs_opt(&apub.summary, slur_regex)?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn from_apub(
    apub: Self::ApubType,
    data: &Self::DataType,
    _request_counter: &mut i32,
  ) -> Result<Self, LemmyError> {
    let domain = apub.id.inner().domain().expect("group id has domain");
    let instance = DbInstance::read_or_create(data.pool(), domain.to_string()).await?;

    let site_form = SiteInsertForm {
      name: apub.name.clone(),
      sidebar: read_from_string_or_source_opt(&apub.content, &None, &apub.source),
      updated: apub.updated.map(|u| u.clone().naive_local()),
      icon: apub.icon.clone().map(|i| i.url.into()),
      banner: apub.image.clone().map(|i| i.url.into()),
      description: apub.summary.clone(),
      actor_id: Some(apub.id.clone().into()),
      last_refreshed_at: Some(naive_now()),
      inbox_url: Some(apub.inbox.clone().into()),
      public_key: Some(apub.public_key.public_key_pem.clone()),
      private_key: None,
      instance_id: instance.id,
    };
    let languages = LanguageTag::to_language_id_multiple(apub.language, data.pool()).await?;

    let site = Site::create(data.pool(), &site_form).await?;
    SiteLanguage::update(data.pool(), languages, &site).await?;
    Ok(site.into())
  }
}

impl ActorType for ApubSite {
  fn actor_id(&self) -> Url {
    self.actor_id.clone().into()
  }
  fn private_key(&self) -> Option<String> {
    self.private_key.clone()
  }
}

impl Actor for ApubSite {
  fn public_key(&self) -> &str {
    &self.public_key
  }

  fn inbox(&self) -> Url {
    self.inbox_url.clone().into()
  }
}

/// Try to fetch the instance actor (to make things like instance rules available).
pub(in crate::objects) async fn fetch_instance_actor_for_object<T: Into<Url> + Clone>(
  object_id: &T,
  context: &LemmyContext,
  request_counter: &mut i32,
) -> Result<InstanceId, LemmyError> {
  let object_id: Url = object_id.clone().into();
  let instance_id = Site::instance_actor_id_from_url(object_id);
  let site = ObjectId::<ApubSite>::new(instance_id.clone())
    .dereference(context, local_instance(context).await, request_counter)
    .await;
  match site {
    Ok(s) => Ok(s.instance_id),
    Err(e) => {
      // Failed to fetch instance actor, its probably not a lemmy instance
      debug!("Failed to dereference site for {}: {}", &instance_id, e);
      let domain = instance_id.domain().expect("has domain");
      Ok(
        DbInstance::read_or_create(context.pool(), domain.to_string())
          .await?
          .id,
      )
    }
  }
}

pub(crate) async fn remote_instance_inboxes(pool: &DbPool) -> Result<Vec<Url>, LemmyError> {
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
  use super::*;
  use crate::{objects::tests::init_context, protocol::tests::file_to_json_object};
  use lemmy_db_schema::traits::Crud;
  use serial_test::serial;

  pub(crate) async fn parse_lemmy_instance(context: &LemmyContext) -> ApubSite {
    let json: Instance = file_to_json_object("assets/lemmy/objects/instance.json").unwrap();
    let id = Url::parse("https://enterprise.lemmy.ml/").unwrap();
    let mut request_counter = 0;
    ApubSite::verify(&json, &id, context, &mut request_counter)
      .await
      .unwrap();
    let site = ApubSite::from_apub(json, context, &mut request_counter)
      .await
      .unwrap();
    assert_eq!(request_counter, 0);
    site
  }

  #[actix_rt::test]
  #[serial]
  async fn test_parse_lemmy_instance() {
    let context = init_context().await;
    let site = parse_lemmy_instance(&context).await;

    assert_eq!(site.name, "Enterprise");
    assert_eq!(site.description.as_ref().unwrap().len(), 15);

    Site::delete(context.pool(), site.id).await.unwrap();
  }
}
