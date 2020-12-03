use crate::{
  extensions::context::lemmy_context,
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
use activitystreams_ext::Ext1;
use anyhow::Context;
use lemmy_db::{
  naive_now,
  user::{UserForm, User_},
  ApubObject,
  DbPool,
};
use lemmy_structs::blocking;
use lemmy_utils::{
  location_info,
  settings::Settings,
  utils::{check_slurs, check_slurs_opt, convert_datetime},
  LemmyError,
};
use lemmy_websocket::LemmyContext;
use url::Url;

#[async_trait::async_trait(?Send)]
impl ToApub for User_ {
  type ApubType = PersonExt;

  async fn to_apub(&self, _pool: &DbPool) -> Result<PersonExt, LemmyError> {
    let mut person = ApObject::new(Person::new());
    person
      .set_many_contexts(lemmy_context()?)
      .set_id(Url::parse(&self.actor_id)?)
      .set_published(convert_datetime(self.published));

    if let Some(u) = self.updated {
      person.set_updated(convert_datetime(u));
    }

    if let Some(avatar_url) = &self.avatar {
      let mut image = Image::new();
      image.set_url(Url::parse(avatar_url)?);
      person.set_icon(image.into_any_base()?);
    }

    if let Some(banner_url) = &self.banner {
      let mut image = Image::new();
      image.set_url(Url::parse(banner_url)?);
      person.set_image(image.into_any_base()?);
    }

    if let Some(bio) = &self.bio {
      set_content_and_source(&mut person, bio)?;
      // Also set summary for compatibility with older Lemmy versions.
      // TODO: remove this after a while.
      person.set_summary(bio.to_owned());
    }

    if let Some(i) = self.preferred_username.to_owned() {
      person.set_name(i);
    }

    let mut ap_actor = ApActor::new(self.get_inbox_url()?, person);
    ap_actor
      .set_preferred_username(self.name.to_owned())
      .set_outbox(self.get_outbox_url()?)
      .set_endpoints(Endpoints {
        shared_inbox: Some(self.get_shared_inbox_url()?),
        ..Default::default()
      });

    Ok(Ext1::new(ap_actor, self.get_public_key_ext()?))
  }
  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    unimplemented!()
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for User_ {
  type ApubType = PersonExt;

  async fn from_apub(
    person: &PersonExt,
    context: &LemmyContext,
    expected_domain: Url,
    request_counter: &mut i32,
  ) -> Result<User_, LemmyError> {
    let user_id = person.id_unchecked().context(location_info!())?.to_owned();
    let domain = user_id.domain().context(location_info!())?;
    if domain == Settings::get().hostname {
      let user = blocking(context.pool(), move |conn| {
        User_::read_from_apub_id(conn, user_id.as_str())
      })
      .await??;
      Ok(user)
    } else {
      let user_form =
        UserForm::from_apub(person, context, expected_domain, request_counter).await?;
      let user = blocking(context.pool(), move |conn| User_::upsert(conn, &user_form)).await??;
      Ok(user)
    }
  }
}

#[async_trait::async_trait(?Send)]
impl FromApubToForm<PersonExt> for UserForm {
  async fn from_apub(
    person: &PersonExt,
    _context: &LemmyContext,
    expected_domain: Url,
    _request_counter: &mut i32,
  ) -> Result<Self, LemmyError> {
    let avatar = match person.icon() {
      Some(any_image) => Some(
        Image::from_any_base(any_image.as_one().context(location_info!())?.clone())?
          .context(location_info!())?
          .url()
          .context(location_info!())?
          .as_single_xsd_any_uri()
          .map(|u| u.to_string()),
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
          .map(|u| u.to_string()),
      ),
      None => None,
    };

    let name: String = person
      .inner
      .preferred_username()
      .context(location_info!())?
      .to_string();
    let preferred_username: Option<String> = person
      .name()
      .map(|n| n.one())
      .flatten()
      .map(|n| n.to_owned().xsd_string())
      .flatten();

    let bio = get_source_markdown_value(person)?;

    check_slurs(&name)?;
    check_slurs_opt(&preferred_username)?;
    check_slurs_opt(&bio)?;

    Ok(UserForm {
      name,
      preferred_username: Some(preferred_username),
      password_encrypted: "".to_string(),
      admin: false,
      banned: None,
      email: None,
      avatar,
      banner,
      published: person.inner.published().map(|u| u.to_owned().naive_local()),
      updated: person.updated().map(|u| u.to_owned().naive_local()),
      show_nsfw: false,
      theme: "".to_string(),
      default_sort_type: 0,
      default_listing_type: 0,
      lang: "".to_string(),
      show_avatars: false,
      send_notifications_to_email: false,
      matrix_user_id: None,
      actor_id: Some(check_object_domain(person, expected_domain)?),
      bio: Some(bio),
      local: false,
      private_key: None,
      public_key: Some(person.ext_one.public_key.to_owned().public_key_pem),
      last_refreshed_at: Some(naive_now()),
    })
  }
}
