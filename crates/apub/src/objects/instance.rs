use crate::{
  check_apub_id_valid_with_strictness,
  local_instance,
  objects::read_from_string_or_source_opt,
  protocol::{
    objects::instance::{Instance, InstanceType},
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
use chrono::NaiveDateTime;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{
  source::site::{Site, SiteForm},
  utils::{naive_now, DbPool},
};
use lemmy_utils::{
  error::LemmyError,
  utils::{check_slurs, check_slurs_opt, convert_datetime, markdown_to_html},
};
use lemmy_websocket::LemmyContext;
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
      blocking(data.pool(), move |conn| {
        Site::read_from_apub_id(conn, object_id)
      })
      .await??
      .map(Into::into),
    )
  }

  async fn delete(self, _data: &Self::DataType) -> Result<(), LemmyError> {
    unimplemented!()
  }

  #[tracing::instrument(skip_all)]
  async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    let instance = Instance {
      kind: InstanceType::Service,
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
    check_apub_id_valid_with_strictness(apub.id.inner(), true, data.settings())?;
    verify_domains_match(expected_domain, apub.id.inner())?;

    let slur_regex = &data.settings().slur_regex();
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
    let site_form = SiteForm {
      name: apub.name.clone(),
      sidebar: Some(read_from_string_or_source_opt(
        &apub.content,
        &None,
        &apub.source,
      )),
      updated: apub.updated.map(|u| u.clone().naive_local()),
      icon: Some(apub.icon.clone().map(|i| i.url.into())),
      banner: Some(apub.image.clone().map(|i| i.url.into())),
      description: Some(apub.summary.clone()),
      actor_id: Some(apub.id.clone().into()),
      last_refreshed_at: Some(naive_now()),
      inbox_url: Some(apub.inbox.clone().into()),
      public_key: Some(apub.public_key.public_key_pem.clone()),
      ..SiteForm::default()
    };
    let site = blocking(data.pool(), move |conn| Site::upsert(conn, &site_form)).await??;
    Ok(site.into())
  }
}

impl ActorType for ApubSite {
  fn actor_id(&self) -> Url {
    self.actor_id.to_owned().into()
  }
  fn private_key(&self) -> Option<String> {
    self.private_key.to_owned()
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

/// Instance actor is at the root path, so we simply need to clear the path and other unnecessary
/// parts of the url.
pub fn instance_actor_id_from_url(mut url: Url) -> Url {
  url.set_fragment(None);
  url.set_path("");
  url.set_query(None);
  url
}

/// try to fetch the instance actor (to make things like instance rules available)
pub(in crate::objects) async fn fetch_instance_actor_for_object(
  object_id: Url,
  context: &LemmyContext,
  request_counter: &mut i32,
) {
  // try to fetch the instance actor (to make things like instance rules available)
  let instance_id = instance_actor_id_from_url(object_id);
  let site = ObjectId::<ApubSite>::new(instance_id.clone())
    .dereference(context, local_instance(context), request_counter)
    .await;
  if let Err(e) = site {
    debug!("Failed to dereference site for {}: {}", instance_id, e);
  }
}

pub(crate) async fn remote_instance_inboxes(pool: &DbPool) -> Result<Vec<Url>, LemmyError> {
  Ok(
    blocking(pool, Site::read_remote_sites)
      .await??
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
    let context = init_context();
    let site = parse_lemmy_instance(&context).await;

    assert_eq!(site.name, "Enterprise");
    assert_eq!(site.description.as_ref().unwrap().len(), 15);

    Site::delete(&*context.pool().get().unwrap(), site.id).unwrap();
  }
}
