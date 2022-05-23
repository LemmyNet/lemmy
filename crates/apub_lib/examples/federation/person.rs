use crate::{
  activities::{CreateNote, Follow},
  note::MyPost,
};
use activitystreams_kinds::{actor::PersonType, public};
use anyhow::Error;
use lemmy_apub_lib::{
  object_id::ObjectId,
  signatures::{Keypair, PublicKey},
  traits::{ActorType, ApubObject},
};
use lemmy_utils::LemmyError;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(new)]
pub struct MyUser {
  ap_id: ObjectId<MyUser>,
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
  pub(crate) fn followers(&self) -> Result<Url, Error> {
    Ok(Url::parse(&format!("{}/followers", self.ap_id.inner()))?)
  }

  pub async fn follow(&self, other: &MyUser) -> Result<(), Error> {
    let follow = Follow::new(self.ap_id.clone(), other.ap_id.clone(), todo!());
    // TODO: send
    Ok(())
  }

  pub async fn post(&self, post: MyPost) -> Result<(), Error> {
    let create = CreateNote::new(
      self.ap_id.clone(),
      vec![public(), self.followers()?],
      post.into_apub(&()).await?,
      todo!(),
    );
    // TODO: send
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
    object_id: Url,
    data: &Self::DataType,
  ) -> Result<Option<Self>, LemmyError>
  where
    Self: Sized,
  {
    todo!()
  }

  async fn delete(self, data: &Self::DataType) -> Result<(), LemmyError> {
    todo!()
  }

  async fn into_apub(self, data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    todo!()
  }

  fn to_tombstone(&self) -> Result<Self::TombstoneType, LemmyError> {
    todo!()
  }

  async fn verify(
    apub: &Self::ApubType,
    expected_domain: &Url,
    data: &Self::DataType,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    todo!()
  }

  async fn from_apub(
    apub: Self::ApubType,
    data: &Self::DataType,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError>
  where
    Self: Sized,
  {
    todo!()
  }
}

impl ActorType for MyUser {
  fn actor_id(&self) -> Url {
    self.ap_id.clone().into_inner()
  }

  fn public_key(&self) -> String {
    self.keypair.public_key.clone()
  }

  fn private_key(&self) -> Option<String> {
    Some(self.keypair.private_key.clone())
  }

  fn inbox_url(&self) -> Url {
    Url::parse(&format!("{}/inbox", &self.ap_id)).expect("generate inbox url")
  }
}
