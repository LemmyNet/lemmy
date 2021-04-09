use crate::{
  extensions::{context::lemmy_context, person_extension::PersonExtension},
  objects::{
    check_object_domain,
    get_source_markdown_value,
    set_content_and_source,
    FromApub,
    FromApubToForm,
    ToApub,
  },
  ActorType,
  PersonExt,
};
use activitystreams::{
  actor::{ApActor, Endpoints, Person},
  object::{ApObject, Image, Tombstone},
  prelude::*,
};
use activitystreams_ext::Ext2;
use anyhow::Context;
use lemmy_api_common::blocking;
use lemmy_db_queries::{ApubObject, DbPool};
use lemmy_db_schema::{
  naive_now,
  source::person::{Person as DbPerson, PersonForm},
};
use lemmy_utils::{
  location_info,
  settings::structs::Settings,
  utils::{check_slurs, check_slurs_opt, convert_datetime},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use url::Url;

#[async_trait::async_trait(?Send)]
impl ToApub for DbPerson {
  type ApubType = PersonExt;

  async fn to_apub(&self, _pool: &DbPool) -> Result<PersonExt, LemmyError> {
    let mut person = ApObject::new(Person::new());
    person
      .set_many_contexts(lemmy_context()?)
      .set_id(self.actor_id.to_owned().into_inner())
      .set_published(convert_datetime(self.published));

    if let Some(u) = self.updated {
      person.set_updated(convert_datetime(u));
    }

    if let Some(avatar_url) = &self.avatar {
      let mut image = Image::new();
      image.set_url::<Url>(avatar_url.to_owned().into());
      person.set_icon(image.into_any_base()?);
    }

    if let Some(banner_url) = &self.banner {
      let mut image = Image::new();
      image.set_url::<Url>(banner_url.to_owned().into());
      person.set_image(image.into_any_base()?);
    }

    if let Some(bio) = &self.bio {
      set_content_and_source(&mut person, bio)?;
    }

    // In apub, the "name" is a display name
    if let Some(i) = self.display_name.to_owned() {
      person.set_name(i);
    }

    let mut ap_actor = ApActor::new(self.inbox_url.clone().into(), person);
    ap_actor
      .set_preferred_username(self.name.to_owned())
      .set_outbox(self.get_outbox_url()?)
      .set_endpoints(Endpoints {
        shared_inbox: Some(self.get_shared_inbox_or_inbox_url()),
        ..Default::default()
      });

    let person_ext = PersonExtension::new(self.matrix_user_id.to_owned())?;
    Ok(Ext2::new(ap_actor, person_ext, self.get_public_key_ext()?))
  }
  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    unimplemented!()
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for DbPerson {
  type ApubType = PersonExt;

  async fn from_apub(
    person: &PersonExt,
    context: &LemmyContext,
    expected_domain: Url,
    request_counter: &mut i32,
    mod_action_allowed: bool,
  ) -> Result<DbPerson, LemmyError> {
    let person_id = person.id_unchecked().context(location_info!())?.to_owned();
    let domain = person_id.domain().context(location_info!())?;
    if domain == Settings::get().hostname() {
      let person = blocking(context.pool(), move |conn| {
        DbPerson::read_from_apub_id(conn, &person_id.into())
      })
      .await??;
      Ok(person)
    } else {
      let person_form = PersonForm::from_apub(
        person,
        context,
        expected_domain,
        request_counter,
        mod_action_allowed,
      )
      .await?;
      let person = blocking(context.pool(), move |conn| {
        DbPerson::upsert(conn, &person_form)
      })
      .await??;
      Ok(person)
    }
  }
}

#[async_trait::async_trait(?Send)]
impl FromApubToForm<PersonExt> for PersonForm {
  async fn from_apub(
    person: &PersonExt,
    _context: &LemmyContext,
    expected_domain: Url,
    _request_counter: &mut i32,
    _mod_action_allowed: bool,
  ) -> Result<Self, LemmyError> {
    let avatar = match person.icon() {
      Some(any_image) => Some(
        Image::from_any_base(any_image.as_one().context(location_info!())?.clone())?
          .context(location_info!())?
          .url()
          .context(location_info!())?
          .as_single_xsd_any_uri()
          .map(|url| url.to_owned()),
      ),
      None => None,
    };

    let banner = match person.image() {
      Some(any_image) => Some(
        Image::from_any_base(any_image.as_one().context(location_info!())?.clone())
          .context(location_info!())?
          .context(location_info!())?
          .url()
          .context(location_info!())?
          .as_single_xsd_any_uri()
          .map(|url| url.to_owned()),
      ),
      None => None,
    };

    let name: String = person
      .inner
      .preferred_username()
      .context(location_info!())?
      .to_string();
    let display_name: Option<String> = person
      .name()
      .map(|n| n.one())
      .flatten()
      .map(|n| n.to_owned().xsd_string())
      .flatten();
    let bio = get_source_markdown_value(person)?;
    let shared_inbox = person
      .inner
      .endpoints()?
      .map(|e| e.shared_inbox)
      .flatten()
      .map(|s| s.to_owned().into());

    check_slurs(&name)?;
    check_slurs_opt(&display_name)?;
    check_slurs_opt(&bio)?;

    Ok(PersonForm {
      name,
      display_name: Some(display_name),
      banned: None,
      deleted: None,
      avatar: avatar.map(|o| o.map(|i| i.into())),
      banner: banner.map(|o| o.map(|i| i.into())),
      published: person.inner.published().map(|u| u.to_owned().naive_local()),
      updated: person.updated().map(|u| u.to_owned().naive_local()),
      actor_id: Some(check_object_domain(person, expected_domain)?),
      bio: Some(bio),
      local: Some(false),
      admin: Some(false),
      private_key: None,
      public_key: Some(Some(person.ext_two.public_key.to_owned().public_key_pem)),
      last_refreshed_at: Some(naive_now()),
      inbox_url: Some(person.inner.inbox()?.to_owned().into()),
      shared_inbox_url: Some(shared_inbox),
      matrix_user_id: Some(person.ext_one.matrix_user_id.to_owned()),
    })
  }
}
