use crate::{
  check_apub_id_valid_with_strictness,
  generate_outbox_url,
  objects::{instance::fetch_instance_actor_for_object, read_from_string_or_source_opt},
  protocol::{
    objects::{
      person::{Person, UserTypes},
      Endpoints,
    },
    ImageObject,
    Source,
  },
  ActorType,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  traits::{Actor, ApubObject},
  utils::verify_domains_match,
};
use chrono::NaiveDateTime;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{
  source::person::{Person as DbPerson, PersonForm},
  traits::ApubActor,
  utils::naive_now,
};
use lemmy_utils::{
  error::LemmyError,
  utils::{check_slurs, check_slurs_opt, convert_datetime, markdown_to_html},
};
use lemmy_websocket::LemmyContext;
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApubPerson(DbPerson);

impl Deref for ApubPerson {
  type Target = DbPerson;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<DbPerson> for ApubPerson {
  fn from(p: DbPerson) -> Self {
    ApubPerson(p)
  }
}

#[async_trait::async_trait(?Send)]
impl ApubObject for ApubPerson {
  type DataType = LemmyContext;
  type ApubType = Person;
  type DbType = DbPerson;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    Some(self.last_refreshed_at)
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_apub_id(
    object_id: Url,
    context: &LemmyContext,
  ) -> Result<Option<Self>, LemmyError> {
    Ok(
      blocking(context.pool(), move |conn| {
        DbPerson::read_from_apub_id(conn, &object_id.into())
      })
      .await??
      .map(Into::into),
    )
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, context: &LemmyContext) -> Result<(), LemmyError> {
    blocking(context.pool(), move |conn| {
      DbPerson::update_deleted(conn, self.id, true)
    })
    .await??;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn into_apub(self, _pool: &LemmyContext) -> Result<Person, LemmyError> {
    let kind = if self.bot_account {
      UserTypes::Service
    } else {
      UserTypes::Person
    };

    let person = Person {
      kind,
      id: ObjectId::new(self.actor_id.clone()),
      preferred_username: self.name.clone(),
      name: self.display_name.clone(),
      summary: self.bio.as_ref().map(|b| markdown_to_html(b)),
      source: self.bio.clone().map(Source::new),
      icon: self.avatar.clone().map(ImageObject::new),
      image: self.banner.clone().map(ImageObject::new),
      matrix_user_id: self.matrix_user_id.clone(),
      published: Some(convert_datetime(self.published)),
      outbox: generate_outbox_url(&self.actor_id)?.into(),
      endpoints: self.shared_inbox_url.clone().map(|s| Endpoints {
        shared_inbox: s.into(),
      }),
      public_key: self.get_public_key(),
      updated: self.updated.map(convert_datetime),
      inbox: self.inbox_url.clone().into(),
    };
    Ok(person)
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    person: &Person,
    expected_domain: &Url,
    context: &LemmyContext,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_domains_match(person.id.inner(), expected_domain)?;
    check_apub_id_valid_with_strictness(person.id.inner(), false, context.settings())?;

    let slur_regex = &context.settings().slur_regex();
    check_slurs(&person.preferred_username, slur_regex)?;
    check_slurs_opt(&person.name, slur_regex)?;
    let bio = read_from_string_or_source_opt(&person.summary, &None, &person.source);
    check_slurs_opt(&bio, slur_regex)?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn from_apub(
    person: Person,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubPerson, LemmyError> {
    let person_form = PersonForm {
      name: person.preferred_username,
      display_name: Some(person.name),
      banned: None,
      ban_expires: None,
      deleted: None,
      avatar: Some(person.icon.map(|i| i.url.into())),
      banner: Some(person.image.map(|i| i.url.into())),
      published: person.published.map(|u| u.naive_local()),
      updated: person.updated.map(|u| u.naive_local()),
      actor_id: Some(person.id.into()),
      bio: Some(read_from_string_or_source_opt(
        &person.summary,
        &None,
        &person.source,
      )),
      local: Some(false),
      admin: Some(false),
      bot_account: Some(person.kind == UserTypes::Service),
      private_key: None,
      public_key: Some(person.public_key.public_key_pem),
      last_refreshed_at: Some(naive_now()),
      inbox_url: Some(person.inbox.into()),
      shared_inbox_url: Some(person.endpoints.map(|e| e.shared_inbox.into())),
      matrix_user_id: Some(person.matrix_user_id),
    };
    let person = blocking(context.pool(), move |conn| {
      DbPerson::upsert(conn, &person_form)
    })
    .await??;

    let actor_id = person.actor_id.clone().into();
    fetch_instance_actor_for_object(actor_id, context, request_counter).await;

    Ok(person.into())
  }
}

impl ActorType for ApubPerson {
  fn actor_id(&self) -> Url {
    self.actor_id.to_owned().into()
  }

  fn private_key(&self) -> Option<String> {
    self.private_key.to_owned()
  }
}

impl Actor for ApubPerson {
  fn public_key(&self) -> &str {
    &self.public_key
  }

  fn inbox(&self) -> Url {
    self.inbox_url.clone().into()
  }

  fn shared_inbox(&self) -> Option<Url> {
    self.shared_inbox_url.clone().map(|s| s.into())
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::{
    objects::{
      instance::{tests::parse_lemmy_instance, ApubSite},
      tests::init_context,
    },
    protocol::{objects::instance::Instance, tests::file_to_json_object},
  };
  use lemmy_db_schema::{source::site::Site, traits::Crud};
  use serial_test::serial;

  pub(crate) async fn parse_lemmy_person(context: &LemmyContext) -> (ApubPerson, ApubSite) {
    let site = parse_lemmy_instance(context).await;
    let json = file_to_json_object("assets/lemmy/objects/person.json").unwrap();
    let url = Url::parse("https://enterprise.lemmy.ml/u/picard").unwrap();
    let mut request_counter = 0;
    ApubPerson::verify(&json, &url, context, &mut request_counter)
      .await
      .unwrap();
    let person = ApubPerson::from_apub(json, context, &mut request_counter)
      .await
      .unwrap();
    assert_eq!(request_counter, 0);
    (person, site)
  }

  #[actix_rt::test]
  #[serial]
  async fn test_parse_lemmy_person() {
    let context = init_context();
    let conn = &mut context.pool().get().unwrap();
    let (person, site) = parse_lemmy_person(&context).await;

    assert_eq!(person.display_name, Some("Jean-Luc Picard".to_string()));
    assert!(!person.local);
    assert_eq!(person.bio.as_ref().unwrap().len(), 39);

    DbPerson::delete(conn, person.id).unwrap();
    Site::delete(conn, site.id).unwrap();
  }

  #[actix_rt::test]
  #[serial]
  async fn test_parse_pleroma_person() {
    let context = init_context();
    let conn = &mut context.pool().get().unwrap();

    // create and parse a fake pleroma instance actor, to avoid network request during test
    let mut json: Instance = file_to_json_object("assets/lemmy/objects/instance.json").unwrap();
    let id = Url::parse("https://queer.hacktivis.me/").unwrap();
    json.id = ObjectId::new(id);
    let mut request_counter = 0;
    let site = ApubSite::from_apub(json, &context, &mut request_counter)
      .await
      .unwrap();

    let json = file_to_json_object("assets/pleroma/objects/person.json").unwrap();
    let url = Url::parse("https://queer.hacktivis.me/users/lanodan").unwrap();
    let mut request_counter = 0;
    ApubPerson::verify(&json, &url, &context, &mut request_counter)
      .await
      .unwrap();
    let person = ApubPerson::from_apub(json, &context, &mut request_counter)
      .await
      .unwrap();

    assert_eq!(person.actor_id, url.into());
    assert_eq!(person.name, "lanodan");
    assert!(!person.local);
    assert_eq!(request_counter, 0);
    assert_eq!(person.bio.as_ref().unwrap().len(), 873);

    DbPerson::delete(conn, person.id).unwrap();
    Site::delete(conn, site.id).unwrap();
  }
}
