use crate::{
  objects::{community::ApubCommunity, instance::ApubSite},
  protocol::objects::{group::Group, instance::Instance},
  ActorType,
};
use activitypub_federation::{core::object_id::ObjectId, traits::ApubObject};
use chrono::NaiveDateTime;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{source::site::Site, utils::DbPool};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::LemmyContext;
use serde::Deserialize;
use url::Url;

pub mod block_user;
pub mod undo_block_user;

#[derive(Clone, Debug)]
pub enum SiteOrCommunity {
  Site(ApubSite),
  Community(ApubCommunity),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum InstanceOrGroup {
  Instance(Instance),
  Group(Group),
}

#[async_trait::async_trait(?Send)]
impl ApubObject for SiteOrCommunity {
  type DataType = LemmyContext;
  type ApubType = InstanceOrGroup;
  type DbType = ();
  type Error = LemmyError;

  #[tracing::instrument(skip_all)]
  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    Some(match self {
      SiteOrCommunity::Site(i) => i.last_refreshed_at,
      SiteOrCommunity::Community(c) => c.last_refreshed_at,
    })
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_apub_id(
    object_id: Url,
    data: &Self::DataType,
  ) -> Result<Option<Self>, LemmyError>
  where
    Self: Sized,
  {
    let site = ApubSite::read_from_apub_id(object_id.clone(), data).await?;
    Ok(match site {
      Some(o) => Some(SiteOrCommunity::Site(o)),
      None => ApubCommunity::read_from_apub_id(object_id, data)
        .await?
        .map(SiteOrCommunity::Community),
    })
  }

  async fn delete(self, _data: &Self::DataType) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn into_apub(self, _data: &Self::DataType) -> Result<Self::ApubType, LemmyError> {
    unimplemented!()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    apub: &Self::ApubType,
    expected_domain: &Url,
    data: &Self::DataType,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    match apub {
      InstanceOrGroup::Instance(i) => {
        ApubSite::verify(i, expected_domain, data, request_counter).await
      }
      InstanceOrGroup::Group(g) => {
        ApubCommunity::verify(g, expected_domain, data, request_counter).await
      }
    }
  }

  #[tracing::instrument(skip_all)]
  async fn from_apub(
    apub: Self::ApubType,
    data: &Self::DataType,
    request_counter: &mut i32,
  ) -> Result<Self, LemmyError>
  where
    Self: Sized,
  {
    Ok(match apub {
      InstanceOrGroup::Instance(p) => {
        SiteOrCommunity::Site(ApubSite::from_apub(p, data, request_counter).await?)
      }
      InstanceOrGroup::Group(n) => {
        SiteOrCommunity::Community(ApubCommunity::from_apub(n, data, request_counter).await?)
      }
    })
  }
}

impl SiteOrCommunity {
  fn id(&self) -> ObjectId<SiteOrCommunity> {
    match self {
      SiteOrCommunity::Site(s) => ObjectId::new(s.actor_id.clone()),
      SiteOrCommunity::Community(c) => ObjectId::new(c.actor_id.clone()),
    }
  }
}

async fn generate_cc(target: &SiteOrCommunity, pool: &DbPool) -> Result<Vec<Url>, LemmyError> {
  Ok(match target {
    SiteOrCommunity::Site(_) => blocking(pool, Site::read_remote_sites)
      .await??
      .into_iter()
      .map(|s| s.actor_id.into())
      .collect(),
    SiteOrCommunity::Community(c) => vec![c.actor_id()],
  })
}
