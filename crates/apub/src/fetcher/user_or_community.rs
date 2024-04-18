use crate::{
  activities::GetActorType,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::objects::{group::Group, person::Person},
};
use activitypub_federation::{
  config::Data,
  traits::{Actor, Object},
};
use chrono::{DateTime, Utc};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::activity::ActorType;
use lemmy_utils::error::{LemmyError, LemmyResult};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug)]
pub enum UserOrCommunity {
  User(ApubPerson),
  Community(ApubCommunity),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum PersonOrGroup {
  Person(Person),
  Group(Group),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum PersonOrGroupType {
  Person,
  Group,
}

#[async_trait::async_trait]
impl Object for UserOrCommunity {
  type DataType = LemmyContext;
  type Kind = PersonOrGroup;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    Some(match self {
      UserOrCommunity::User(p) => p.last_refreshed_at,
      UserOrCommunity::Community(p) => p.last_refreshed_at,
    })
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_id(object_id: Url, data: &Data<Self::DataType>) -> LemmyResult<Option<Self>> {
    let person = ApubPerson::read_from_id(object_id.clone(), data).await?;
    Ok(match person {
      Some(o) => Some(UserOrCommunity::User(o)),
      None => ApubCommunity::read_from_id(object_id, data)
        .await?
        .map(UserOrCommunity::Community),
    })
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, data: &Data<Self::DataType>) -> LemmyResult<()> {
    match self {
      UserOrCommunity::User(p) => p.delete(data).await,
      UserOrCommunity::Community(p) => p.delete(data).await,
    }
  }

  async fn into_json(self, _data: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    unimplemented!()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    apub: &Self::Kind,
    expected_domain: &Url,
    data: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    match apub {
      PersonOrGroup::Person(a) => ApubPerson::verify(a, expected_domain, data).await,
      PersonOrGroup::Group(a) => ApubCommunity::verify(a, expected_domain, data).await,
    }
  }

  #[tracing::instrument(skip_all)]
  async fn from_json(apub: Self::Kind, data: &Data<Self::DataType>) -> LemmyResult<Self> {
    Ok(match apub {
      PersonOrGroup::Person(p) => UserOrCommunity::User(ApubPerson::from_json(p, data).await?),
      PersonOrGroup::Group(p) => {
        UserOrCommunity::Community(ApubCommunity::from_json(p, data).await?)
      }
    })
  }
}

impl Actor for UserOrCommunity {
  fn id(&self) -> Url {
    match self {
      UserOrCommunity::User(u) => u.id(),
      UserOrCommunity::Community(c) => c.id(),
    }
  }

  fn public_key_pem(&self) -> &str {
    match self {
      UserOrCommunity::User(p) => p.public_key_pem(),
      UserOrCommunity::Community(p) => p.public_key_pem(),
    }
  }

  fn private_key_pem(&self) -> Option<String> {
    match self {
      UserOrCommunity::User(p) => p.private_key_pem(),
      UserOrCommunity::Community(p) => p.private_key_pem(),
    }
  }

  fn inbox(&self) -> Url {
    unimplemented!()
  }
}

impl GetActorType for UserOrCommunity {
  fn actor_type(&self) -> ActorType {
    match self {
      UserOrCommunity::User(p) => p.actor_type(),
      UserOrCommunity::Community(p) => p.actor_type(),
    }
  }
}
