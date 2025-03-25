use crate::{
  fetcher::post_or_comment::{PageOrNote, PostOrComment},
  objects::community::ApubCommunity,
  protocol::objects::group::Group,
};
use activitypub_federation::{config::Data, traits::Object};
use chrono::{DateTime, Utc};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::{LemmyError, LemmyResult};
use reqwest::Url;
use serde::Deserialize;

/// The types of ActivityPub objects that reports can be created for.
#[derive(Debug)]
pub(crate) enum ReportableObjects {
  PostOrComment(PostOrComment),
  Community(ApubCommunity),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum ReportableKinds {
  PageOrNote(PageOrNote),
  Group(Box<Group>),
}

#[async_trait::async_trait]
impl Object for ReportableObjects {
  type DataType = LemmyContext;
  type Kind = ReportableKinds;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    match self {
      ReportableObjects::PostOrComment(p) => p.last_refreshed_at(),
      ReportableObjects::Community(c) => c.last_refreshed_at(),
    }
  }

  async fn read_from_id(object_id: Url, data: &Data<Self::DataType>) -> LemmyResult<Option<Self>> {
    let community = ApubCommunity::read_from_id(object_id.clone(), data).await?;
    Ok(match community {
      Some(o) => Some(ReportableObjects::Community(o)),
      None => PostOrComment::read_from_id(object_id, data)
        .await?
        .map(ReportableObjects::PostOrComment),
    })
  }

  async fn delete(self, data: &Data<Self::DataType>) -> LemmyResult<()> {
    match self {
      ReportableObjects::PostOrComment(p) => p.delete(data).await,
      ReportableObjects::Community(c) => c.delete(data).await,
    }
  }

  async fn into_json(self, data: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    Ok(match self {
      ReportableObjects::PostOrComment(p) => ReportableKinds::PageOrNote(p.into_json(data).await?),
      ReportableObjects::Community(c) => ReportableKinds::Group(Box::new(c.into_json(data).await?)),
    })
  }

  async fn verify(
    apub: &Self::Kind,
    expected_domain: &Url,
    data: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    match apub {
      ReportableKinds::PageOrNote(p) => PostOrComment::verify(p, expected_domain, data).await,
      ReportableKinds::Group(g) => ApubCommunity::verify(g, expected_domain, data).await,
    }
  }

  async fn from_json(apub: Self::Kind, data: &Data<Self::DataType>) -> LemmyResult<Self> {
    Ok(match apub {
      ReportableKinds::PageOrNote(p) => {
        ReportableObjects::PostOrComment(PostOrComment::from_json(p, data).await?)
      }
      ReportableKinds::Group(g) => {
        ReportableObjects::Community(ApubCommunity::from_json(*g, data).await?)
      }
    })
  }
}
