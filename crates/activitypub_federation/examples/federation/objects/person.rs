use crate::{
  activities::{accept::Accept, create_note::CreateNote, follow::Follow},
  error::Error,
  lib::generate_object_id,
  objects::note::MyPost,
  ObjectId,
};
use activitypub_federation::{
  activity_queue::SendActivity,
  context::WithContext,
  inbox::ActorPublicKey,
  signatures::{Keypair, PublicKey},
  traits::{ActivityHandler, ApubObject},
  LocalInstance,
};
use activitypub_federation_derive::activity_handler;
use activitystreams_kinds::{actor::PersonType, public};
use serde::{Deserialize, Serialize};
use url::Url;

pub struct MyUser {
  pub ap_id: ObjectId<MyUser>,
  keypair: Keypair,
  followers: Vec<Url>,
  pub local: bool,
}

/// List of all activities which this actor can receive.
#[activity_handler((), Error)]
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum PersonAcceptedActivities {
  Follow(Follow),
  Accept(Accept),
  CreateNote(CreateNote),
}

impl MyUser {
  pub fn new(ap_id: Url, keypair: Keypair) -> MyUser {
    MyUser {
      ap_id: ObjectId::new(ap_id),
      keypair,
      followers: vec![],
      local: true,
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
  #[serde(rename = "type")]
  kind: PersonType,
  id: Url,
  inbox: Url,
  public_key: PublicKey,
}

impl MyUser {
  pub fn followers(&self) -> &Vec<Url> {
    &self.followers
  }

  pub fn followers_url(&self) -> Result<Url, Error> {
    Ok(Url::parse(&format!("{}/followers", self.ap_id.inner()))?)
  }

  fn public_key(&self) -> PublicKey {
    PublicKey::new_main_key(
      self.ap_id.clone().into_inner(),
      self.keypair.private_key.clone(),
    )
  }

  pub async fn follow(
    &self,
    other: &MyUser,
    local_instance: &LocalInstance,
    hostname: &str,
  ) -> Result<(), Error> {
    let id = generate_object_id(hostname)?;
    let follow = Follow::new(self.ap_id.clone(), other.ap_id.clone(), id.clone());
    self
      .send(
        id,
        follow,
        vec![other.ap_id.clone().into_inner()],
        local_instance,
      )
      .await?;
    Ok(())
  }

  pub async fn post(
    &self,
    post: MyPost,
    local_instance: &LocalInstance,
    hostname: &str,
  ) -> Result<(), Error> {
    let id = generate_object_id(hostname)?;
    let to = vec![public(), self.followers_url()?];
    let create = CreateNote::new(post.into_apub(&()).await?, id.clone());
    self.send(id, &create, to, local_instance).await?;
    Ok(())
  }

  // TODO: maybe store LocalInstance in self
  pub(crate) async fn send<Activity: Serialize>(
    &self,
    activity_id: Url,
    activity: Activity,
    inboxes: Vec<Url>,
    local_instance: &LocalInstance,
  ) -> Result<(), Error> {
    let serialized = serde_json::to_string(&WithContext::new_default(activity))?;
    SendActivity {
      activity_id,
      actor_public_key: self.public_key(),
      actor_private_key: self.keypair.private_key.clone(),
      inboxes,
      activity: serialized,
    }
    .send(local_instance)
    .await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl ApubObject for MyUser {
  type DataType = ();
  type ApubType = Person;
  type DbType = MyUser;
  type TombstoneType = ();
  type Error = crate::error::Error;

  async fn read_from_apub_id(
    _object_id: Url,
    _data: &Self::DataType,
  ) -> Result<Option<Self>, Self::Error>
  where
    Self: Sized,
  {
    todo!()
  }

  async fn delete(self, _data: &Self::DataType) -> Result<(), Self::Error> {
    todo!()
  }

  async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, Self::Error> {
    todo!()
  }

  fn to_tombstone(&self) -> Result<Self::TombstoneType, Self::Error> {
    todo!()
  }

  async fn verify(
    _apub: &Self::ApubType,
    _expected_domain: &Url,
    _data: &Self::DataType,
    _request_counter: &mut i32,
  ) -> Result<(), Self::Error> {
    todo!()
  }

  async fn from_apub(
    _apub: Self::ApubType,
    _data: &Self::DataType,
    _request_counter: &mut i32,
  ) -> Result<Self, Self::Error>
  where
    Self: Sized,
  {
    todo!()
  }
}

impl ActorPublicKey for MyUser {
  fn public_key(&self) -> &str {
    &self.keypair.public_key
  }
}
