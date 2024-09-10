use crate::{
  objects::{comment::ApubComment, community::ApubCommunity, post::ApubPost},
  protocol::{
    objects::{note::Note, page::Page},
    InCommunity,
  },
};
use activitypub_federation::{config::Data, traits::Object};
use chrono::{DateTime, Utc};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  source::{community::Community, post::Post},
  traits::Crud,
};
use lemmy_utils::{
  error::{LemmyError, LemmyResult},
  LemmyErrorType,
};
use serde::Deserialize;
use url::Url;

#[derive(Clone, Debug)]
pub enum PostOrComment {
  Post(ApubPost),
  Comment(ApubComment),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum PageOrNote {
  Page(Box<Page>),
  Note(Note),
}

#[async_trait::async_trait]
impl Object for PostOrComment {
  type DataType = LemmyContext;
  type Kind = PageOrNote;
  type Error = LemmyError;

  fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
    None
  }

  #[tracing::instrument(skip_all)]
  async fn read_from_id(object_id: Url, data: &Data<Self::DataType>) -> LemmyResult<Option<Self>> {
    let post = ApubPost::read_from_id(object_id.clone(), data).await?;
    Ok(match post {
      Some(o) => Some(PostOrComment::Post(o)),
      None => ApubComment::read_from_id(object_id, data)
        .await?
        .map(PostOrComment::Comment),
    })
  }

  #[tracing::instrument(skip_all)]
  async fn delete(self, data: &Data<Self::DataType>) -> LemmyResult<()> {
    match self {
      PostOrComment::Post(p) => p.delete(data).await,
      PostOrComment::Comment(c) => c.delete(data).await,
    }
  }

  async fn into_json(self, data: &Data<Self::DataType>) -> LemmyResult<Self::Kind> {
    Ok(match self {
      PostOrComment::Post(p) => PageOrNote::Page(Box::new(p.into_json(data).await?)),
      PostOrComment::Comment(c) => PageOrNote::Note(c.into_json(data).await?),
    })
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    apub: &Self::Kind,
    expected_domain: &Url,
    data: &Data<Self::DataType>,
  ) -> LemmyResult<()> {
    match apub {
      PageOrNote::Page(a) => ApubPost::verify(a, expected_domain, data).await,
      PageOrNote::Note(a) => ApubComment::verify(a, expected_domain, data).await,
    }
  }

  #[tracing::instrument(skip_all)]
  async fn from_json(apub: PageOrNote, context: &Data<LemmyContext>) -> LemmyResult<Self> {
    Ok(match apub {
      PageOrNote::Page(p) => PostOrComment::Post(ApubPost::from_json(*p, context).await?),
      PageOrNote::Note(n) => PostOrComment::Comment(ApubComment::from_json(n, context).await?),
    })
  }
}

#[async_trait::async_trait]
impl InCommunity for PostOrComment {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let cid = match self {
      PostOrComment::Post(p) => p.community_id,
      PostOrComment::Comment(c) => {
        Post::read(&mut context.pool(), c.post_id)
          .await?
          .ok_or(LemmyErrorType::CouldntFindPost)?
          .community_id
      }
    };
    Ok(
      Community::read(&mut context.pool(), cid)
        .await?
        .ok_or(LemmyErrorType::CouldntFindCommunity)?
        .into(),
    )
  }
}
