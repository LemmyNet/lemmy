use crate::{
  objects::instance::fetch_instance_actor_for_object,
  protocol::person::{Person, UserTypes},
  utils::{
    functions::{
      check_apub_id_valid_with_strictness,
      read_from_string_or_source_opt,
      GetActorType,
    },
    markdown_links::markdown_rewrite_remote_links_opt,
    protocol::{ImageObject, Source},
  },
};
use activitypub_federation::{
  config::Data,
  protocol::verification::{verify_domains_match, verify_is_remote_object},
  traits::{Actor, Object},
};
use chrono::{DateTime, Utc};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{
    generate_outbox_url,
    get_url_blocklist,
    process_markdown_opt,
    proxy_image_link_opt_apub,
    slur_regex,
  },
};
use lemmy_db_schema::{
  sensitive::SensitiveString,
  source::person::{Person as DbPerson, PersonInsertForm, PersonUpdateForm},
  traits::{ApubActor, Crud},
};
use lemmy_db_schema_file::enums::ActorType;
use lemmy_utils::{
  error::{LemmyError, LemmyResult},
  utils::{
    markdown::markdown_to_html,
    slurs::{check_slurs, check_slurs_opt},
  },
};
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApubPerson(pub DbPerson);

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

#[async_trait::async_trait]
impl Object for ApubPerson {
  type DataType = LemmyContext;
  type Kind = Person;
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
      DbPerson::read_from_apub_id(&mut context.pool(), &object_id.into())
        .await?
        .map(Into::into),
    )
  }

  async fn delete(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let form = PersonUpdateForm {
      deleted: Some(true),
      ..Default::default()
    };
    DbPerson::update(&mut context.pool(), self.id, &form).await?;
    Ok(())
  }

  fn is_deleted(&self) -> bool {
    self.deleted
  }

  async fn into_json(self, _context: &Data<Self::DataType>) -> LemmyResult<Person> {
    let kind = if self.bot_account {
      UserTypes::Service
    } else {
      UserTypes::Person
    };

    let person = Person {
      kind,
      id: self.ap_id.clone().into(),
      preferred_username: self.name.clone(),
      name: self.display_name.clone(),
      summary: self.bio.as_ref().map(|b| markdown_to_html(b)),
      source: self.bio.clone().map(Source::new),
      icon: self.avatar.clone().map(ImageObject::new),
      image: self.banner.clone().map(ImageObject::new),
      matrix_user_id: self.matrix_user_id.clone(),
      published: Some(self.published_at),
      outbox: generate_outbox_url(&self.ap_id)?.into(),
      endpoints: None,
      public_key: self.public_key(),
      updated: self.updated_at,
      inbox: self.inbox_url.clone().into(),
    };
    Ok(person)
  }

  async fn verify(
    person: &Person,
    expected_domain: &Url,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    let slur_regex = slur_regex(context).await?;
    check_slurs(&person.preferred_username, &slur_regex)?;
    check_slurs_opt(&person.name, &slur_regex)?;

    verify_domains_match(person.id.inner(), expected_domain)?;
    verify_is_remote_object(&person.id, context)?;
    check_apub_id_valid_with_strictness(person.id.inner(), false, context).await?;

    let bio = read_from_string_or_source_opt(&person.summary, &None, &person.source);
    check_slurs_opt(&bio, &slur_regex)?;
    Ok(())
  }

  async fn from_json(person: Person, context: &Data<Self::DataType>) -> LemmyResult<ApubPerson> {
    let instance_id = fetch_instance_actor_for_object(&person.id, context).await?;

    let slur_regex = slur_regex(context).await?;
    let url_blocklist = get_url_blocklist(context).await?;
    let bio = read_from_string_or_source_opt(&person.summary, &None, &person.source);
    let bio = process_markdown_opt(&bio, &slur_regex, &url_blocklist, context).await?;
    let bio = markdown_rewrite_remote_links_opt(bio, context).await;
    let avatar = proxy_image_link_opt_apub(person.icon.map(|i| i.url), context).await?;
    let banner = proxy_image_link_opt_apub(person.image.map(|i| i.url), context).await?;

    // Some Mastodon users have `name: ""` (empty string), need to convert that to `None`
    // https://github.com/mastodon/mastodon/issues/25233
    let display_name = person.name.filter(|n| !n.is_empty());

    let person_form = PersonInsertForm {
      name: person.preferred_username,
      display_name,
      deleted: Some(false),
      avatar,
      banner,
      published_at: person.published,
      updated_at: person.updated,
      ap_id: Some(person.id.into()),
      bio,
      local: Some(false),
      bot_account: Some(person.kind == UserTypes::Service),
      private_key: None,
      public_key: person.public_key.public_key_pem,
      last_refreshed_at: Some(Utc::now()),
      inbox_url: Some(
        person
          .endpoints
          .map(|e| e.shared_inbox)
          .unwrap_or(person.inbox)
          .into(),
      ),
      matrix_user_id: person.matrix_user_id,
      instance_id,
    };
    let person = DbPerson::upsert(&mut context.pool(), &person_form).await?;

    Ok(person.into())
  }
}

impl Actor for ApubPerson {
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

impl GetActorType for ApubPerson {
  fn actor_type(&self) -> ActorType {
    ActorType::Person
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::{
    objects::instance::ApubSite,
    protocol::instance::Instance,
    utils::test::{file_to_json_object, parse_lemmy_person},
  };
  use activitypub_federation::fetch::object_id::ObjectId;
  use lemmy_db_schema::source::site::Site;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_parse_lemmy_person() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;
    let (person, site) = parse_lemmy_person(&context).await?;

    assert_eq!(person.display_name, Some("Jean-Luc Picard".to_string()));
    assert!(!person.local);
    assert_eq!(person.bio.as_ref().map(std::string::String::len), Some(39));

    cleanup((person, site), &context).await?;
    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_parse_pleroma_person() -> LemmyResult<()> {
    let context = LemmyContext::init_test_context().await;

    // create and parse a fake pleroma instance actor, to avoid network request during test
    let mut json: Instance = file_to_json_object("../apub/assets/lemmy/objects/instance.json")?;
    json.id = ObjectId::parse("https://queer.hacktivis.me/")?;
    let url = Url::parse("https://queer.hacktivis.me/users/lanodan")?;
    ApubSite::verify(&json, &url, &context).await?;
    let site = ApubSite::from_json(json, &context).await?;

    let json = file_to_json_object("../apub/assets/pleroma/objects/person.json")?;
    ApubPerson::verify(&json, &url, &context).await?;
    let person = ApubPerson::from_json(json, &context).await?;

    assert_eq!(person.ap_id, url.into());
    assert_eq!(person.name, "lanodan");
    assert!(!person.local);
    assert_eq!(context.request_count(), 0);
    assert_eq!(person.bio.as_ref().map(std::string::String::len), Some(812));

    cleanup((person, site), &context).await?;
    Ok(())
  }

  async fn cleanup(
    (person, site): (ApubPerson, ApubSite),
    context: &LemmyContext,
  ) -> LemmyResult<()> {
    DbPerson::delete(&mut context.pool(), person.id).await?;
    Site::delete(&mut context.pool(), site.id).await?;
    Ok(())
  }
}
