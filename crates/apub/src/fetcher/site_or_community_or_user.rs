use crate::{
  fetcher::user_or_community::{PersonOrGroup, UserOrCommunity},
  objects::instance::ApubSite,
  protocol::objects::instance::Instance,
};
use activitypub_federation::{
  config::Data,
  traits::{Actor, Object},
};
use chrono::{DateTime, Utc};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::{LemmyError, LemmyResult};
use reqwest::Url;
use serde::{Deserialize, Serialize};

// todo: maybe this enum should be somewhere else?
#[derive(Debug)]
pub enum SiteOrCommunityOrUser {
  Site(ApubSite),
  UserOrCommunity(UserOrCommunity),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum SiteOrPersonOrGroup {
  Instance(Instance),
  PersonOrGroup(PersonOrGroup),
}

#[async_trait::async_trait]
impl Object for SiteOrCommunityOrUser {
  type DataType = LemmyContext;
  type Kind = SiteOrPersonOrGroup;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    Some(match self {
      SiteOrCommunityOrUser::Site(p) => p.last_refreshed_at,
      SiteOrCommunityOrUser::UserOrCommunity(p) => p.last_refreshed_at()?,
    })
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_id(
    _object_id: Url,
    _data: &Data<Self::DataType>,
  ) -> LemmyResult<Option<Self>> {
    unimplemented!();
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, data: &Data<Self::DataType>) -> LemmyResult<()> {
    match self {
      SiteOrCommunityOrUser::Site(p) => p.delete(data).await,
      SiteOrCommunityOrUser::UserOrCommunity(p) => p.delete(data).await,
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
      SiteOrPersonOrGroup::Instance(a) => ApubSite::verify(a, expected_domain, data).await,
      SiteOrPersonOrGroup::PersonOrGroup(a) => {
        UserOrCommunity::verify(a, expected_domain, data).await
      }
    }
  }

  #[tracing::instrument(skip_all)]
  async fn from_json(_apub: Self::Kind, _data: &Data<Self::DataType>) -> LemmyResult<Self> {
    unimplemented!();
  }
}

impl Actor for SiteOrCommunityOrUser {
  fn id(&self) -> Url {
    match self {
      SiteOrCommunityOrUser::Site(u) => u.id(),
      SiteOrCommunityOrUser::UserOrCommunity(c) => c.id(),
    }
  }

  fn public_key_pem(&self) -> &str {
    match self {
      SiteOrCommunityOrUser::Site(p) => p.public_key_pem(),
      SiteOrCommunityOrUser::UserOrCommunity(p) => p.public_key_pem(),
    }
  }

  fn private_key_pem(&self) -> Option<String> {
    match self {
      SiteOrCommunityOrUser::Site(p) => p.private_key_pem(),
      SiteOrCommunityOrUser::UserOrCommunity(p) => p.private_key_pem(),
    }
  }

  fn inbox(&self) -> Url {
    unimplemented!()
  }
}
