use crate::{
  apub::{
    activities::send_activity,
    create_apub_response,
    extensions::signatures::PublicKey,
    ActorType,
    FromApub,
    PersonExt,
    ToApub,
  },
  blocking,
  convert_datetime,
  db::{
    activity::insert_activity,
    user::{UserForm, User_},
  },
  naive_now,
  routes::DbPoolParam,
  DbPool,
  LemmyError,
};
use activitystreams::{
  actor::{properties::ApActorProperties, Person},
  context,
  endpoint::EndpointProperties,
  object::{properties::ObjectProperties, AnyImage, Image},
  primitives::XsdAnyUri,
};
use activitystreams_ext::Ext2;
use activitystreams_new::{
  activity::{Follow, Undo},
  object::Tombstone,
  prelude::*,
};
use actix_web::{body::Body, client::Client, web, HttpResponse};
use serde::Deserialize;

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
    let mut person = Person::default();
    let oprops: &mut ObjectProperties = person.as_mut();
    oprops
      .set_context_xsd_any_uri(context())?
      .set_id(self.actor_id.to_string())?
      .set_name_xsd_string(self.name.to_owned())?
      .set_published(convert_datetime(self.published))?;

    if let Some(u) = self.updated {
      oprops.set_updated(convert_datetime(u))?;
    }

    if let Some(i) = &self.preferred_username {
      oprops.set_name_xsd_string(i.to_owned())?;
    }

    if let Some(avatar_url) = &self.avatar {
      let mut image = Image::new();
      image
        .object_props
        .set_url_xsd_any_uri(avatar_url.to_owned())?;
      let any_image = AnyImage::from_concrete(image)?;
      oprops.set_icon_any_image(any_image)?;
    }

    let mut endpoint_props = EndpointProperties::default();

    endpoint_props.set_shared_inbox(self.get_shared_inbox_url())?;

    let mut actor_props = ApActorProperties::default();

    actor_props
      .set_inbox(self.get_inbox_url())?
      .set_outbox(self.get_outbox_url())?
      .set_endpoints(endpoint_props)?
      .set_followers(self.get_followers_url())?
      .set_following(self.get_following_url())?
      .set_liked(self.get_liked_url())?;

    Ok(Ext2::new(person, actor_props, self.get_public_key_ext()))
  }
  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    unimplemented!()
  }
}

#[async_trait::async_trait(?Send)]
impl ActorType for User_ {
  fn actor_id(&self) -> String {
    self.actor_id.to_owned()
  }

  fn public_key(&self) -> String {
    self.public_key.to_owned().unwrap()
  }

  fn private_key(&self) -> String {
    self.private_key.to_owned().unwrap()
  }

  /// As a given local user, send out a follow request to a remote community.
  async fn send_follow(
    &self,
    follow_actor_id: &str,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let id = format!("{}/follow/{}", self.actor_id, uuid::Uuid::new_v4());
    let mut follow = Follow::new(self.actor_id.to_owned(), follow_actor_id);
    follow.set_context(context()).set_id(id.parse()?);
    let to = format!("{}/inbox", follow_actor_id);

    insert_activity(self.id, follow.clone(), true, pool).await?;

    send_activity(client, &follow, self, vec![to]).await?;
    Ok(())
  }

  async fn send_unfollow(
    &self,
    follow_actor_id: &str,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let id = format!("{}/follow/{}", self.actor_id, uuid::Uuid::new_v4());
    let mut follow = Follow::new(self.actor_id.to_owned(), follow_actor_id);
    follow.set_context(context()).set_id(id.parse()?);

    let to = format!("{}/inbox", follow_actor_id);

    // TODO
    // Undo that fake activity
    let undo_id = format!("{}/undo/follow/{}", self.actor_id, uuid::Uuid::new_v4());
    let mut undo = Undo::new(self.actor_id.parse::<XsdAnyUri>()?, follow.into_any_base()?);
    undo.set_context(context()).set_id(undo_id.parse()?);

    insert_activity(self.id, undo.clone(), true, pool).await?;

    send_activity(client, &undo, self, vec![to]).await?;
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
    _follow: &Follow,
    _client: &Client,
    _pool: &DbPool,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn get_follower_inboxes(&self, _pool: &DbPool) -> Result<Vec<String>, LemmyError> {
    unimplemented!()
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for UserForm {
  type ApubType = PersonExt;
  /// Parse an ActivityPub person received from another instance into a Lemmy user.
  async fn from_apub(person: &PersonExt, _: &Client, _: &DbPool) -> Result<Self, LemmyError> {
    let oprops = &person.inner.object_props;
    let aprops = &person.ext_one;
    let public_key: &PublicKey = &person.ext_two.public_key;

    let avatar = match oprops.get_icon_any_image() {
      Some(any_image) => any_image
        .to_owned()
        .into_concrete::<Image>()?
        .object_props
        .get_url_xsd_any_uri()
        .map(|u| u.to_string()),
      None => None,
    };

    Ok(UserForm {
      name: oprops.get_name_xsd_string().unwrap().to_string(),
      preferred_username: aprops.get_preferred_username().map(|u| u.to_string()),
      password_encrypted: "".to_string(),
      admin: false,
      banned: false,
      email: None,
      avatar,
      updated: oprops
        .get_updated()
        .map(|u| u.as_ref().to_owned().naive_local()),
      show_nsfw: false,
      theme: "".to_string(),
      default_sort_type: 0,
      default_listing_type: 0,
      lang: "".to_string(),
      show_avatars: false,
      send_notifications_to_email: false,
      matrix_user_id: None,
      actor_id: oprops.get_id().unwrap().to_string(),
      bio: oprops.get_summary_xsd_string().map(|s| s.to_string()),
      local: false,
      private_key: None,
      public_key: Some(public_key.to_owned().public_key_pem),
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
