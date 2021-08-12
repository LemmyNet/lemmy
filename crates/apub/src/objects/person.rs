use crate::{
  check_is_apub_id_valid,
  extensions::{context::lemmy_context, signatures::PublicKey},
  objects::{FromApub, ImageObject, Source, ToApub},
  ActorType,
};
use activitystreams::{
  actor::Endpoints,
  base::AnyBase,
  chrono::{DateTime, FixedOffset},
  object::{kind::ImageType, Tombstone},
  primitives::OneOrMany,
  unparsed::Unparsed,
};
use lemmy_api_common::blocking;
use lemmy_apub_lib::{
  values::{MediaTypeHtml, MediaTypeMarkdown},
  verify_domains_match,
};
use lemmy_db_queries::{ApubObject, DbPool};
use lemmy_db_schema::{
  naive_now,
  source::person::{Person as DbPerson, PersonForm},
};
use lemmy_utils::{
  utils::{check_slurs, check_slurs_opt, convert_datetime, markdown_to_html},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use url::Url;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub enum UserTypes {
  Person,
  Service,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
  #[serde(rename = "@context")]
  context: OneOrMany<AnyBase>,
  #[serde(rename = "type")]
  kind: UserTypes,
  id: Url,
  /// username, set at account creation and can never be changed
  preferred_username: String,
  /// displayname (can be changed at any time)
  name: Option<String>,
  content: Option<String>,
  media_type: Option<MediaTypeHtml>,
  source: Option<Source>,
  /// user avatar
  icon: Option<ImageObject>,
  /// user banner
  image: Option<ImageObject>,
  matrix_user_id: Option<String>,
  inbox: Url,
  /// mandatory field in activitypub, currently empty in lemmy
  outbox: Url,
  endpoints: Endpoints<Url>,
  public_key: PublicKey,
  published: DateTime<FixedOffset>,
  updated: Option<DateTime<FixedOffset>>,
  #[serde(flatten)]
  unparsed: Unparsed,
}

// TODO: can generate this with a derive macro
impl Person {
  pub(crate) fn id(&self, expected_domain: &Url) -> Result<&Url, LemmyError> {
    verify_domains_match(&self.id, expected_domain)?;
    Ok(&self.id)
  }
}

#[async_trait::async_trait(?Send)]
impl ToApub for DbPerson {
  type ApubType = Person;

  async fn to_apub(&self, _pool: &DbPool) -> Result<Person, LemmyError> {
    let kind = if self.bot_account {
      UserTypes::Service
    } else {
      UserTypes::Person
    };
    let source = self.bio.clone().map(|bio| Source {
      content: bio,
      media_type: MediaTypeMarkdown::Markdown,
    });
    let icon = self.avatar.clone().map(|url| ImageObject {
      kind: ImageType::Image,
      url: url.into(),
    });
    let image = self.banner.clone().map(|url| ImageObject {
      kind: ImageType::Image,
      url: url.into(),
    });

    let person = Person {
      context: lemmy_context(),
      kind,
      id: self.actor_id.to_owned().into_inner(),
      preferred_username: self.name.clone(),
      name: self.display_name.clone(),
      content: self.bio.as_ref().map(|b| markdown_to_html(b)),
      media_type: self.bio.as_ref().map(|_| MediaTypeHtml::Html),
      source,
      icon,
      image,
      matrix_user_id: self.matrix_user_id.clone(),
      published: convert_datetime(self.published),
      outbox: self.get_outbox_url()?,
      endpoints: Endpoints {
        shared_inbox: self.shared_inbox_url.clone().map(|s| s.into()),
        ..Default::default()
      },
      public_key: self.get_public_key()?,
      updated: self.updated.map(convert_datetime),
      unparsed: Default::default(),
      inbox: self.inbox_url.clone().into(),
    };
    Ok(person)
  }
  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    unimplemented!()
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for DbPerson {
  type ApubType = Person;

  async fn from_apub(
    person: &Person,
    context: &LemmyContext,
    expected_domain: &Url,
    _request_counter: &mut i32,
  ) -> Result<DbPerson, LemmyError> {
    let actor_id = Some(person.id(expected_domain)?.clone().into());
    let name = person.preferred_username.clone();
    let display_name: Option<String> = person.name.clone();
    let bio = person.source.clone().map(|s| s.content);
    let shared_inbox = person.endpoints.shared_inbox.clone().map(|s| s.into());
    let bot_account = match person.kind {
      UserTypes::Person => false,
      UserTypes::Service => true,
    };

    check_slurs(&name)?;
    check_slurs_opt(&display_name)?;
    check_slurs_opt(&bio)?;
    check_is_apub_id_valid(&person.id, false)?;

    let person_form = PersonForm {
      name,
      display_name: Some(display_name),
      banned: None,
      deleted: None,
      avatar: Some(person.icon.clone().map(|i| i.url.into())),
      banner: Some(person.image.clone().map(|i| i.url.into())),
      published: Some(person.published.naive_local()),
      updated: person.updated.map(|u| u.clone().naive_local()),
      actor_id,
      bio: Some(bio),
      local: Some(false),
      admin: Some(false),
      bot_account: Some(bot_account),
      private_key: None,
      public_key: Some(Some(person.public_key.public_key_pem.clone())),
      last_refreshed_at: Some(naive_now()),
      inbox_url: Some(person.inbox.to_owned().into()),
      shared_inbox_url: Some(shared_inbox),
      matrix_user_id: Some(person.matrix_user_id.clone()),
    };
    let person = blocking(context.pool(), move |conn| {
      DbPerson::upsert(conn, &person_form)
    })
    .await??;
    Ok(person)
  }
}
