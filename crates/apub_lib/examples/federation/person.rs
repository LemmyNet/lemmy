use crate::{
  activities::{CreateNote, Follow},
  lib::generate_object_id,
  note::MyPost,
};
use activitystreams_kinds::{actor::PersonType, public};
use anyhow::Error;
use lemmy_apub_lib::{
  activity_queue::SendActivity,
  context::WithContext,
  object_id::ObjectId,
  signatures::{Keypair, PublicKey},
  traits::ApubObject,
  LocalInstance,
};
use lemmy_utils::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(new)]
pub struct MyUser {
  pub ap_id: ObjectId<MyUser>,
  keypair: Keypair,
  #[new(default)]
  followers: Vec<Url>,
  #[new(default)]
  pub known_posts: Vec<MyPost>,
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
  ) -> Result<(), LemmyError> {
    let id = generate_object_id(hostname)?;
    let to = vec![public(), self.followers_url()?];
    let create = CreateNote::new(
      self.ap_id.clone(),
      to.clone(),
      post.into_apub(&()).await?,
      id.clone(),
    );
    self.send(id, &create, to, local_instance).await?;
    Ok(())
  }

  // TODO: maybe store LocalInstance in self
  async fn send<Activity: Serialize>(
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

  async fn read_from_apub_id(
    _object_id: Url,
    _data: &Self::DataType,
  ) -> Result<Option<Self>, LemmyError>
  where
    Self: Sized,
  {
    todo!()
  }

  async fn delete(self, _data: &Self::DataType) -> Result<(), LemmyError> {
    todo!()
  }

  async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    todo!()
  }

  fn to_tombstone(&self) -> Result<Self::TombstoneType, LemmyError> {
    todo!()
  }

  async fn verify(
    _apub: &Self::ApubType,
    _expected_domain: &Url,
    _data: &Self::DataType,
    _request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    todo!()
  }

  async fn from_apub(
    _apub: Self::ApubType,
    _data: &Self::DataType,
    _request_counter: &mut i32,
  ) -> Result<Self, LemmyError>
  where
    Self: Sized,
  {
    todo!()
  }
}
