use super::verify_is_remote_object;
use crate::{
  activities::GetActorType,
  check_apub_id_valid_with_strictness,
  local_site_data_cached,
  objects::{instance::fetch_instance_actor_for_object, read_from_string_or_source_opt},
  protocol::{
    objects::{
      person::{Person, UserTypes},
      Endpoints,
    },
    ImageObject,
    Source,
  },
};
use activitypub_federation::{
  config::Data,
  protocol::verification::verify_domains_match,
  traits::{Actor, Object},
};
use chrono::{DateTime, Utc};
use lemmy_api_common::{
  context::LemmyContext,
  utils::{
    generate_outbox_url,
    get_url_blocklist,
    local_site_opt_to_slur_regex,
    process_markdown_opt,
    proxy_image_link_opt_apub,
  },
};
use lemmy_db_schema::{
  source::{
    activity::ActorType,
    local_site::LocalSite,
    person::{Person as DbPerson, PersonInsertForm, PersonUpdateForm},
  },
  traits::{ApubActor, Crud},
  utils::naive_now,
};
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
pub struct ApubPerson(pub(crate) DbPerson);

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

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    Some(self.last_refreshed_at)
  }

  #[tracing::instrument(skip_all)]
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

  #[tracing::instrument(skip_all)]
  async fn delete(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let form = PersonUpdateForm {
      deleted: Some(true),
      ..Default::default()
    };
    DbPerson::update(&mut context.pool(), self.id, &form).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn into_json(self, _context: &Data<Self::DataType>) -> LemmyResult<Person> {
    let kind = if self.bot_account {
      UserTypes::Service
    } else {
      UserTypes::Person
    };

    let person = Person {
      kind,
      id: self.actor_id.clone().into(),
      preferred_username: self.name.clone(),
      name: self.display_name.clone(),
      summary: self.bio.as_ref().map(|b| markdown_to_html(b)),
      source: self.bio.clone().map(Source::new),
      icon: self.avatar.clone().map(ImageObject::new),
      image: self.banner.clone().map(ImageObject::new),
      matrix_user_id: self.matrix_user_id.clone(),
      published: Some(self.published),
      outbox: generate_outbox_url(&self.actor_id)?.into(),
      endpoints: self.shared_inbox_url.clone().map(|s| Endpoints {
        shared_inbox: s.into(),
      }),
      public_key: self.public_key(),
      updated: self.updated,
      inbox: self.inbox_url.clone().into(),
    };
    Ok(person)
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    person: &Person,
    expected_domain: &Url,
    context: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    let local_site_data = local_site_data_cached(&mut context.pool()).await?;
    let slur_regex = &local_site_opt_to_slur_regex(&local_site_data.local_site);
    check_slurs(&person.preferred_username, slur_regex)?;
    check_slurs_opt(&person.name, slur_regex)?;

    verify_domains_match(person.id.inner(), expected_domain)?;
    verify_is_remote_object(&person.id, context)?;
    check_apub_id_valid_with_strictness(person.id.inner(), false, context).await?;

    let bio = read_from_string_or_source_opt(&person.summary, &None, &person.source);
    check_slurs_opt(&bio, slur_regex)?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn from_json(person: Person, context: &Data<Self::DataType>) -> LemmyResult<ApubPerson> {
    let instance_id = fetch_instance_actor_for_object(&person.id, context).await?;

    let local_site = LocalSite::read(&mut context.pool()).await.ok();
    let slur_regex = &local_site_opt_to_slur_regex(&local_site);
    let url_blocklist = get_url_blocklist(context).await?;
    let bio = read_from_string_or_source_opt(&person.summary, &None, &person.source);
    let bio = process_markdown_opt(&bio, slur_regex, &url_blocklist, context).await?;
    let avatar = proxy_image_link_opt_apub(person.icon.map(|i| i.url), context).await?;
    let banner = proxy_image_link_opt_apub(person.image.map(|i| i.url), context).await?;

    // Some Mastodon users have `name: ""` (empty string), need to convert that to `None`
    // https://github.com/mastodon/mastodon/issues/25233
    let display_name = person.name.filter(|n| !n.is_empty());

    let person_form = PersonInsertForm {
      name: person.preferred_username,
      display_name,
      banned: None,
      ban_expires: None,
      deleted: Some(false),
      avatar,
      banner,
      published: person.published.map(Into::into),
      updated: person.updated.map(Into::into),
      actor_id: Some(person.id.into()),
      bio,
      local: Some(false),
      bot_account: Some(person.kind == UserTypes::Service),
      private_key: None,
      public_key: person.public_key.public_key_pem,
      last_refreshed_at: Some(naive_now()),
      inbox_url: Some(person.inbox.into()),
      shared_inbox_url: person.endpoints.map(|e| e.shared_inbox.into()),
      matrix_user_id: person.matrix_user_id,
      instance_id,
    };
    let person = DbPerson::upsert(&mut context.pool(), &person_form).await?;

    Ok(person.into())
  }
}

impl Actor for ApubPerson {
  fn id(&self) -> Url {
    self.actor_id.inner().clone()
  }

  fn public_key_pem(&self) -> &str {
    &self.public_key
  }

  fn private_key_pem(&self) -> Option<String> {
    self.private_key.clone().map(|p| p.into_inner())
  }

  fn inbox(&self) -> Url {
    self.inbox_url.clone().into()
  }

  fn shared_inbox(&self) -> Option<Url> {
    self.shared_inbox_url.clone().map(Into::into)
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
    objects::instance::{tests::parse_lemmy_instance, ApubSite},
    protocol::{objects::instance::Instance, tests::file_to_json_object},
  };
  use activitypub_federation::fetch::object_id::ObjectId;
  use lemmy_db_schema::source::site::Site;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  pub(crate) async fn parse_lemmy_person(
    context: &Data<LemmyContext>,
  ) -> LemmyResult<(ApubPerson, ApubSite)> {
    let site = parse_lemmy_instance(context).await?;
    let json = file_to_json_object("assets/lemmy/objects/person.json")?;
    let url = Url::parse("https://enterprise.lemmy.ml/u/picard")?;
    ApubPerson::verify(&json, &url, context).await?;
    let person = ApubPerson::from_json(json, context).await?;
    assert_eq!(context.request_count(), 0);
    Ok((person, site))
  }

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
    let mut json: Instance = file_to_json_object("assets/lemmy/objects/instance.json")?;
    json.id = ObjectId::parse("https://queer.hacktivis.me/")?;
    let url = Url::parse("https://queer.hacktivis.me/users/lanodan")?;
    ApubSite::verify(&json, &url, &context).await?;
    let site = ApubSite::from_json(json, &context).await?;

    let json = file_to_json_object("assets/pleroma/objects/person.json")?;
    ApubPerson::verify(&json, &url, &context).await?;
    let person = ApubPerson::from_json(json, &context).await?;

    assert_eq!(person.actor_id, url.into());
    assert_eq!(person.name, "lanodan");
    assert!(!person.local);
    assert_eq!(context.request_count(), 0);
    assert_eq!(person.bio.as_ref().map(std::string::String::len), Some(873));

    cleanup((person, site), &context).await?;
    Ok(())
  }

  async fn cleanup(data: (ApubPerson, ApubSite), context: &LemmyContext) -> LemmyResult<()> {
    DbPerson::delete(&mut context.pool(), data.0.id).await?;
    Site::delete(&mut context.pool(), data.1.id).await?;
    Ok(())
  }
}
