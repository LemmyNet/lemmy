use crate::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::objects::{group::Group, person::Person},
};
use chrono::NaiveDateTime;
use lemmy_apub_lib::traits::{ActorType, ApubObject};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;
use url::Url;

#[derive(Clone, Debug)]
pub enum UserOrCommunity {
  User(ApubPerson),
  Community(ApubCommunity),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum PersonOrGroup {
  Person(Person),
  Group(Group),
}

#[async_trait::async_trait(?Send)]
impl ApubObject for UserOrCommunity {
  type DataType = LemmyContext;
  type ApubType = PersonOrGroup;
  type TombstoneType = ();

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    Some(match self {
      UserOrCommunity::User(p) => p.last_refreshed_at,
      UserOrCommunity::Community(p) => p.last_refreshed_at,
    })
  }

  async fn read_from_apub_id(
    object_id: Url,
    data: &Self::DataType,
  ) -> Result<Option<Self>, LemmyError> {
    let person = ApubPerson::read_from_apub_id(object_id.clone(), data).await?;
    Ok(match person {
      Some(o) => Some(UserOrCommunity::User(o)),
      None => ApubCommunity::read_from_apub_id(object_id, data)
        .await?
        .map(UserOrCommunity::Community),
    })
  }

  async fn delete(self, data: &Self::DataType) -> Result<(), LemmyError> {
    match self {
      UserOrCommunity::User(p) => p.delete(data).await,
      UserOrCommunity::Community(p) => p.delete(data).await,
    }
  }

  async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    unimplemented!()
  }

  fn to_tombstone(&self) -> Result<Self::TombstoneType, LemmyError> {
    unimplemented!()
  }

  async fn verify(
    apub: &Self::ApubType,
    expected_domain: &Url,
    data: &Self::DataType,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    match apub {
      PersonOrGroup::Person(a) => {
        ApubPerson::verify(a, expected_domain, data, request_counter).await
      }
      PersonOrGroup::Group(a) => {
        ApubCommunity::verify(a, expected_domain, data, request_counter).await
      }
    }
  }

  async fn from_apub(
    apub: Self::ApubType,
    data: &Self::DataType,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError> {
    Ok(match apub {
      PersonOrGroup::Person(p) => {
        UserOrCommunity::User(ApubPerson::from_apub(p, data, request_counter).await?)
      }
      PersonOrGroup::Group(p) => {
        UserOrCommunity::Community(ApubCommunity::from_apub(p, data, request_counter).await?)
      }
    })
  }
}

impl ActorType for UserOrCommunity {
  fn actor_id(&self) -> Url {
    match self {
      UserOrCommunity::User(p) => p.actor_id(),
      UserOrCommunity::Community(p) => p.actor_id(),
    }
  }

  fn public_key(&self) -> Option<String> {
    match self {
      UserOrCommunity::User(p) => p.public_key(),
      UserOrCommunity::Community(p) => p.public_key(),
    }
  }

  fn private_key(&self) -> Option<String> {
    todo!()
  }

  fn inbox_url(&self) -> Url {
    todo!()
  }

  fn shared_inbox_url(&self) -> Option<Url> {
    todo!()
  }
}
