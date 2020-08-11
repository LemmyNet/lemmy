use crate::{
  api::{check_slurs, check_slurs_opt},
  apub::{
    activities::{generate_activity_id, send_activity},
    check_actor_domain,
    create_apub_response,
    fetcher::get_or_fetch_and_upsert_actor,
    insert_activity,
    ActorType,
    FromApub,
    PersonExt,
    ToApub,
  },
  blocking,
  routes::DbPoolParam,
  DbPool,
  LemmyError,
};
use activitystreams::{
  activity::{
    kind::{FollowType, UndoType},
    Follow,
    Undo,
  },
  actor::{ApActor, Endpoints, Person},
  context,
  object::{Image, Tombstone},
  prelude::*,
};
use activitystreams_ext::Ext1;
use actix_web::{body::Body, client::Client, web, HttpResponse};
use anyhow::Context;
use lemmy_db::{
  naive_now,
  user::{UserForm, User_},
};
use lemmy_utils::{convert_datetime, location_info};
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct UserQuery {
  user_name: String,
}

#[async_trait::async_trait(?Send)]
impl ToApub for User_ {
  type Response = PersonExt;

  // Turn a Lemmy Community into an ActivityPub group that can be sent out over the network.
  async fn to_apub(&self, _pool: &DbPool) -> Result<PersonExt, LemmyError> {
    // TODO go through all these to_string and to_owned()
    let mut person = Person::new();
    person
      .set_context(context())
      .set_id(Url::parse(&self.actor_id)?)
      .set_name(self.name.to_owned())
      .set_published(convert_datetime(self.published));

    if let Some(u) = self.updated {
      person.set_updated(convert_datetime(u));
    }

    if let Some(avatar_url) = &self.avatar {
      let mut image = Image::new();
      image.set_url(avatar_url.to_owned());
      person.set_icon(image.into_any_base()?);
    }

    if let Some(banner_url) = &self.banner {
      let mut image = Image::new();
      image.set_url(banner_url.to_owned());
      person.set_image(image.into_any_base()?);
    }

    if let Some(bio) = &self.bio {
      person.set_summary(bio.to_owned());
    }

    let mut ap_actor = ApActor::new(self.get_inbox_url()?, person);
    ap_actor
      .set_outbox(self.get_outbox_url()?)
      .set_followers(self.get_followers_url().parse()?)
      .set_following(self.get_following_url().parse()?)
      .set_liked(self.get_liked_url().parse()?)
      .set_endpoints(Endpoints {
        shared_inbox: Some(self.get_shared_inbox_url()?),
        ..Default::default()
      });

    if let Some(i) = &self.preferred_username {
      ap_actor.set_preferred_username(i.to_owned());
    }

    Ok(Ext1::new(ap_actor, self.get_public_key_ext()?))
  }
  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    unimplemented!()
  }
}

#[async_trait::async_trait(?Send)]
impl ActorType for User_ {
  fn actor_id_str(&self) -> String {
    self.actor_id.to_owned()
  }

  fn public_key(&self) -> Option<String> {
    self.public_key.to_owned()
  }

  fn private_key(&self) -> Option<String> {
    self.private_key.to_owned()
  }

  /// As a given local user, send out a follow request to a remote community.
  async fn send_follow(
    &self,
    follow_actor_id: &Url,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let mut follow = Follow::new(self.actor_id.to_owned(), follow_actor_id.as_str());
    follow
      .set_context(context())
      .set_id(generate_activity_id(FollowType::Follow)?);
    let follow_actor = get_or_fetch_and_upsert_actor(follow_actor_id, client, pool).await?;
    let to = follow_actor.get_inbox_url()?;

    insert_activity(self.id, follow.clone(), true, pool).await?;

    send_activity(client, &follow.into_any_base()?, self, vec![to]).await?;
    Ok(())
  }

  async fn send_unfollow(
    &self,
    follow_actor_id: &Url,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let mut follow = Follow::new(self.actor_id.to_owned(), follow_actor_id.as_str());
    follow
      .set_context(context())
      .set_id(generate_activity_id(FollowType::Follow)?);
    let follow_actor = get_or_fetch_and_upsert_actor(follow_actor_id, client, pool).await?;

    let to = follow_actor.get_inbox_url()?;

    // Undo that fake activity
    let mut undo = Undo::new(Url::parse(&self.actor_id)?, follow.into_any_base()?);
    undo
      .set_context(context())
      .set_id(generate_activity_id(UndoType::Undo)?);

    insert_activity(self.id, undo.clone(), true, pool).await?;

    send_activity(client, &undo.into_any_base()?, self, vec![to]).await?;
    Ok(())
  }

  async fn send_delete(
    &self,
    _creator: &User_,
    _client: &Client,
    _pool: &DbPool,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_undo_delete(
    &self,
    _creator: &User_,
    _client: &Client,
    _pool: &DbPool,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_remove(
    &self,
    _creator: &User_,
    _client: &Client,
    _pool: &DbPool,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_undo_remove(
    &self,
    _creator: &User_,
    _client: &Client,
    _pool: &DbPool,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_accept_follow(
    &self,
    _follow: Follow,
    _client: &Client,
    _pool: &DbPool,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn get_follower_inboxes(&self, _pool: &DbPool) -> Result<Vec<Url>, LemmyError> {
    unimplemented!()
  }

  fn user_id(&self) -> i32 {
    self.id
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for UserForm {
  type ApubType = PersonExt;
  /// Parse an ActivityPub person received from another instance into a Lemmy user.
  async fn from_apub(
    person: &PersonExt,
    _: &Client,
    _: &DbPool,
    expected_domain: Option<Url>,
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

    let name = person
      .name()
      .context(location_info!())?
      .one()
      .context(location_info!())?
      .as_xsd_string()
      .context(location_info!())?
      .to_string();
    let preferred_username = person.inner.preferred_username().map(|u| u.to_string());
    let bio = person
      .inner
      .summary()
      .map(|s| s.as_single_xsd_string())
      .flatten()
      .map(|s| s.to_string());
    check_slurs(&name)?;
    check_slurs_opt(&preferred_username)?;
    check_slurs_opt(&bio)?;

    Ok(UserForm {
      name,
      preferred_username,
      password_encrypted: "".to_string(),
      admin: false,
      banned: false,
      email: None,
      avatar,
      banner,
      updated: person.updated().map(|u| u.to_owned().naive_local()),
      show_nsfw: false,
      theme: "".to_string(),
      default_sort_type: 0,
      default_listing_type: 0,
      lang: "".to_string(),
      show_avatars: false,
      send_notifications_to_email: false,
      matrix_user_id: None,
      actor_id: check_actor_domain(person, expected_domain)?,
      bio,
      local: false,
      private_key: None,
      public_key: Some(person.ext_one.public_key.to_owned().public_key_pem),
      last_refreshed_at: Some(naive_now()),
    })
  }
}

/// Return the user json over HTTP.
pub async fn get_apub_user_http(
  info: web::Path<UserQuery>,
  db: DbPoolParam,
) -> Result<HttpResponse<Body>, LemmyError> {
  let user_name = info.into_inner().user_name;
  let user = blocking(&db, move |conn| {
    User_::find_by_email_or_username(conn, &user_name)
  })
  .await??;
  let u = user.to_apub(&db).await?;
  Ok(create_apub_response(&u))
}
